pub(crate) mod environment;
mod error;
pub(crate) mod exit_status;

use std::{cell::RefCell, collections::HashMap, path::PathBuf, process::Stdio, rc::Rc, vec};

use crate::{
    ast::{
        AndOr, AndOrPart, AssignmentWord, CmdPrefix, CmdSuffix, Command, CompleteCommand,
        CompleteCommands, List, ListPart, PipeSequence, Pipeline, Program, SeparatorOp,
        SimpleCommand, Word, Wordlist,
    },
    builtin::builtin,
    options::Options,
    token::{Expression, Unit},
};

use self::{environment::ExecutionEnvironment, error::ExecError, exit_status::ExitStatus};

pub struct Executor {
    env: Rc<RefCell<ExecutionEnvironment>>,
    options: Rc<RefCell<Options>>,
}

impl Executor {
    pub fn new(options: Rc<RefCell<Options>>) -> Self {
        Self {
            env: Rc::new(RefCell::new(ExecutionEnvironment::default())),
            options,
        }
    }

    pub fn execute(&self, program: Program) -> Result<ExitStatus, ExecError> {
        let Program(CompleteCommands(complete_commands)) = program;
        let mut status = ExitStatus::SUCCESS;
        for complete_command in complete_commands {
            status = self.execute_complete_command(complete_command)?;
        }
        Ok(status)
    }

    fn execute_complete_command(
        &self,
        complete_command: CompleteCommand,
    ) -> Result<ExitStatus, ExecError> {
        let CompleteCommand(List(list_parts), optional_separator) = complete_command;
        let mut list_part_iterator = list_parts.iter();

        let mut current = list_part_iterator.next();
        let mut status = ExitStatus::SUCCESS;
        loop {
            let next = list_part_iterator.next();

            let separator_op = next.map_or_else(
                || optional_separator.unwrap_or(crate::ast::SeparatorOp::Serial),
                |list_part| match list_part {
                    ListPart::Tail(_, sep) => *sep,
                    _ => unreachable!(),
                },
            );

            status = match current {
                Some(ListPart::Start(and_or)) => self.execute_and_or(and_or, &separator_op)?,
                Some(ListPart::Tail(and_or, _)) => self.execute_and_or(and_or, &separator_op)?,
                None => break,
            };

            current = next;
        }

        Ok(status)
    }

    fn execute_and_or(
        &self,
        and_or: &AndOr,
        _separator_op: &SeparatorOp,
    ) -> Result<ExitStatus, ExecError> {
        let AndOr(parts) = and_or;
        let mut part_iterator = parts.iter();
        let mut status = match part_iterator.next() {
            Some(AndOrPart::Start(pipeline)) => self.execute_pipeline(pipeline)?,
            _ => return Err(ExecError::MalformedPipeline),
        };

        for part in part_iterator {
            status = match part {
                AndOrPart::Start(_) => return Err(ExecError::MalformedPipeline),
                AndOrPart::And(pipeline) if status.is_success() => {
                    self.execute_pipeline(pipeline)?
                }
                AndOrPart::Or(pipeline) if !status.is_success() => {
                    self.execute_pipeline(pipeline)?
                }
                _ => return Ok(status),
            };
        }

        Ok(status)
    }

    fn execute_pipeline(&self, pipeline: &Pipeline) -> Result<ExitStatus, ExecError> {
        // TODO: Handle bang vs normal.
        let status = match pipeline {
            Pipeline::Bang(pipe_sequence) => self.execute_pipe_sequence(pipe_sequence)?,
            Pipeline::Normal(pipe_sequence) => self.execute_pipe_sequence(pipe_sequence)?,
        };

        Ok(status)
    }

    fn execute_pipe_sequence(&self, pipe_sequence: &PipeSequence) -> Result<ExitStatus, ExecError> {
        let PipeSequence(commands) = pipe_sequence;
        let mut status = ExitStatus::SUCCESS;
        for command in commands {
            status = match command {
                Command::Simple(simple_command) => self.execute_simple_command(simple_command)?,
            };
        }

        Ok(status)
    }

    fn execute_simple_command(
        &self,
        simple_command: &SimpleCommand,
    ) -> Result<ExitStatus, ExecError> {
        // TODO: Handle redirects.
        let SimpleCommand(maybe_prefix, maybe_command_name, maybe_suffix) = simple_command;
        if let Some(command_name) = maybe_command_name {
            let expanded_command_name = self.expand_word(command_name);
            let envs = maybe_prefix.as_ref().map_or_else(HashMap::new, |prefix| {
                let CmdPrefix(assignments, _) = prefix;
                assignments
                    .iter()
                    .map(|AssignmentWord(key, value)| (key, value))
                    .collect()
            });
            let arguments = maybe_suffix.as_ref().map_or_else(Vec::new, |suffix| {
                let CmdSuffix(Wordlist(words), _) = suffix;
                let argument_list: Vec<String> =
                    words.iter().map(|word| self.expand_word(word)).collect();
                argument_list
            });

            if let Some(builtin) = builtin(&expanded_command_name) {
                return Ok(builtin.execute(&arguments, &mut self.env.borrow_mut()));
            }

            match expanded_command_name.as_str() {
                "set" => {
                    let command_args: Vec<&str> = arguments.iter().map(AsRef::as_ref).collect();
                    match command_args.as_slice() {
                        ["-o", "xlex"] => self.options.borrow_mut().debug_lexing = true,
                        ["-o", "xparse"] => self.options.borrow_mut().debug_parsing = true,
                        ["-v"] | ["-o", "verbose"] => self.options.borrow_mut().print_input = true,
                        args => {
                            eprintln!("set: unknown arguments {:?}", args);
                            return Ok(ExitStatus::new(1));
                        }
                    }
                    Ok(ExitStatus::SUCCESS)
                }
                program => {
                    let program_path = self.find_program(program);
                    let result = std::process::Command::new(program_path)
                        .args(arguments)
                        .envs(envs)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status();

                    match result {
                        Ok(status) => Ok(ExitStatus::new(status.code().unwrap())),
                        Err(_) => Err(ExecError::UnknownCommand(expanded_command_name)),
                    }
                }
            }
        } else {
            if let Some(CmdPrefix(assignments, _)) = maybe_prefix {
                if assignments.is_empty() {
                    return Err(ExecError::MissingCommand);
                }

                for AssignmentWord(name, value) in assignments {
                    self.env
                        .borrow_mut()
                        .set_var(name.to_owned(), value.to_owned());
                }
            }

            Ok(ExitStatus::SUCCESS)
        }
    }

    fn expand_word(&self, word: &Word) -> String {
        let mut expanded_word = String::new();
        let Word(units) = word;

        for unit in units {
            match unit {
                Unit::Literal(literal) => expanded_word.push_str(&literal),
                Unit::Expression(Expression::AssignDefaultValues(var, default, unset_or_null)) => {
                    let mut env = self.env.borrow_mut();
                    match env.var(&var) {
                        None => {
                            env.set_var(var.to_string(), default.to_string());
                            expanded_word.push_str(&default)
                        }
                        Some(str) if str.is_empty() && !*unset_or_null => (),
                        Some(str) if str.is_empty() && *unset_or_null => {
                            env.set_var(var.to_string(), default.to_string());
                            expanded_word.push_str(&default)
                        }
                        Some(value) => expanded_word.push_str(value),
                    }
                }
                Unit::Expression(Expression::Parameter(var)) | Unit::Var(var) => {
                    match self.env.borrow().var(var) {
                        Some(value) => expanded_word.push_str(value),
                        None if self.options.borrow().allow_unset_vars => (),
                        _ => todo!("exit shell with error"),
                    }
                }
                Unit::Expression(Expression::UseDefaultValues(var, default, unset_or_null)) => {
                    match self.env.borrow().var(&var) {
                        None => expanded_word.push_str(&default),
                        Some(str) if str.is_empty() && !*unset_or_null => (),
                        Some(str) if str.is_empty() && *unset_or_null => {
                            expanded_word.push_str(&default)
                        }
                        Some(value) => expanded_word.push_str(value),
                    }
                }
                _ => unimplemented!("Undefined expansion for unit {:?}", unit),
            }
        }

        expanded_word
    }

    /// Finds a program's path by searching through the `PATH` environment variable.
    /// Returns the program name if no relevant path is found.
    fn find_program(&self, program: &str) -> String {
        // TODO: Refactor into trait.
        // TODO: PATHEXT is Windows-specific.
        let env = self.env.borrow();
        if let Some(path_env) = env.var("PATH") {
            let mut extensions = vec![""];
            if let Some(ext_env) = env.var("PATHEXT") {
                extensions.extend(ext_env.split(';'));
            }

            for path in path_env.split(';') {
                for extension in &extensions {
                    // TODO: Use colon on non-Windows system or translate semicolon to colon on Windows.
                    let program_path: PathBuf =
                        [path, &(program.to_string() + extension)].iter().collect();
                    if program_path.exists() {
                        return program_path.to_string_lossy().to_string();
                    }
                }
            }
        }

        program.to_string()
    }
}
