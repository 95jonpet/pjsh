pub(crate) mod environment;
mod error;
pub(crate) mod exit_status;

use std::{cell::RefCell, collections::HashMap, process::Stdio, rc::Rc};

use crate::{
    ast::{
        AndOr, AndOrPart, AssignmentWord, CmdPrefix, CmdSuffix, Command, CompleteCommand,
        CompleteCommands, List, ListPart, PipeSequence, Pipeline, Program, SeparatorOp,
        SimpleCommand, Word, Wordlist,
    },
    options::Options,
    token::{Expression, Unit},
};

use self::{
    environment::{Environment, WindowsEnvironment},
    error::ExecError,
    exit_status::ExitStatus,
};

pub struct Executor {
    env: Rc<RefCell<dyn Environment>>,
    options: Rc<RefCell<Options>>,
}

impl Executor {
    pub fn new(options: Rc<RefCell<Options>>) -> Self {
        Self {
            env: Rc::new(RefCell::new(WindowsEnvironment::default())),
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

            match expanded_command_name.as_str() {
                // Builtins.
                // TODO: Add builtins.
                // "cd" => Ok((builtin::io::Cd {}).execute(&arguments, &mut self.env)),
                // "exit" => Ok((builtin::io::Exit {}).execute(&arguments, &mut, env)),
                // "false" => Ok((builtin::logic::False {}).execute(&arguments, &mut, env)),
                // "true" => Ok((builtin::logic::True {}).execute(&arguments, &mut, env)),
                // "unset" => Ok((builtin::io::Unset {}).execute(&arguments, &mut, env)),
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

                "which" => {
                    let command_args: Vec<&str> = arguments.iter().map(AsRef::as_ref).collect();
                    match command_args.as_slice() {
                        [program] => {
                            if let Some(path) = self.env.borrow().find_program(program) {
                                let mut pretty_path = path.to_string_lossy().to_string();
                                pretty_path = pretty_path.trim_start_matches(r#"\\?\"#).to_string();
                                println!("{}", pretty_path);
                                Ok(ExitStatus::SUCCESS)
                            } else {
                                eprintln!(
                                    "which: no {} in ({})",
                                    program,
                                    self.env.borrow().var("PATH").unwrap_or_default()
                                );
                                Ok(ExitStatus::new(1))
                            }
                        }
                        args => {
                            eprintln!("set: unknown arguments {:?}", args);
                            Ok(ExitStatus::new(1))
                        }
                    }
                }

                program => {
                    let program_path = self
                        .env
                        .borrow()
                        .find_program(program)
                        .map(|path| path.to_string_lossy().to_string())
                        .unwrap_or_else(|| program.to_string());
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
        // TODO: Return Result<String, String>.
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
                        Some(value) => expanded_word.push_str(&value),
                    }
                }
                Unit::Expression(Expression::IndicateError(var, message, unset_or_null)) => {
                    match self.env.borrow().var(&var) {
                        None => {
                            eprintln!("pjsh: {}: {}", var, message);
                            // TODO: Exit with non-success code.
                        }
                        Some(str) if str.is_empty() && !*unset_or_null => (),
                        Some(str) if str.is_empty() && *unset_or_null => {
                            eprintln!("pjsh: {}: {}", var, message);
                            // TODO: Exit with non-success code.
                        }
                        Some(value) => expanded_word.push_str(&value),
                    }
                }
                Unit::Expression(Expression::Parameter(var)) | Unit::Var(var) => {
                    match self.env.borrow().var(var) {
                        Some(value) => expanded_word.push_str(&value),
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
                        Some(value) => expanded_word.push_str(&value),
                    }
                }
                _ => unimplemented!("Undefined expansion for unit {:?}", unit),
            }
        }

        expanded_word
    }
}
