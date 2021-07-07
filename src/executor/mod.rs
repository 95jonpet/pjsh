use std::{collections::HashMap, process::Stdio};

use crate::ast::{
    AndOr, AndOrPart, AssignmentWord, CmdPrefix, CmdSuffix, Command, CompleteCommand,
    CompleteCommands, List, ListPart, PipeSequence, Pipeline, Program, SeparatorOp, SimpleCommand,
    Word, Wordlist,
};

pub struct ExecError;

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&self, program: Program) -> Result<(), ExecError> {
        let Program(CompleteCommands(complete_commands)) = program;
        for complete_command in complete_commands {
            self.execute_complete_command(complete_command)?;
        }
        Ok(())
    }

    fn execute_complete_command(&self, complete_command: CompleteCommand) -> Result<(), ExecError> {
        let CompleteCommand(List(list_parts), optional_separator) = complete_command;
        let mut list_part_iterator = list_parts.iter();

        let mut current = list_part_iterator.next();
        // let mut next = None;
        loop {
            let next = list_part_iterator.next();

            let separator_op = next.map_or_else(
                || optional_separator.unwrap_or(crate::ast::SeparatorOp::Serial),
                |list_part| match list_part {
                    ListPart::Tail(_, sep) => *sep,
                    _ => unreachable!(),
                },
            );

            match current {
                Some(ListPart::Start(and_or)) => {
                    self.execute_and_or(and_or, &separator_op)?;
                }
                Some(ListPart::Tail(and_or, _)) => {
                    self.execute_and_or(and_or, &separator_op)?;
                }
                _ => unreachable!(),
            };

            if next == None {
                break;
            }

            current = next;
        }

        // The current has not been executed yet.

        Ok(())
    }

    fn execute_and_or(&self, and_or: &AndOr, separator_op: &SeparatorOp) -> Result<(), ExecError> {
        let AndOr(parts) = and_or;
        for part in parts {
            // TODO: Handle and/or logic.
            match part {
                AndOrPart::Start(pipeline) => self.execute_pipeline(pipeline)?,
                AndOrPart::And(pipeline) => self.execute_pipeline(pipeline)?,
                AndOrPart::Or(pipeline) => self.execute_pipeline(pipeline)?,
            }
        }

        Ok(())
    }

    fn execute_pipeline(&self, pipeline: &Pipeline) -> Result<(), ExecError> {
        // TODO: Handle bang vs normal.
        match pipeline {
            Pipeline::Bang(pipe_sequence) => self.execute_pipe_sequence(pipe_sequence)?,
            Pipeline::Normal(pipe_sequence) => self.execute_pipe_sequence(pipe_sequence)?,
        }

        Ok(())
    }

    fn execute_pipe_sequence(&self, pipe_sequence: &PipeSequence) -> Result<(), ExecError> {
        let PipeSequence(commands) = pipe_sequence;
        for command in commands {
            match command {
                Command::Simple(simple_command) => self.execute_simple_command(simple_command)?,
            }
        }

        Ok(())
    }

    fn execute_simple_command(&self, simple_command: &SimpleCommand) -> Result<(), ExecError> {
        // TODO: Handle redirects.
        let SimpleCommand(maybe_prefix, maybe_command_name, maybe_suffix) = simple_command;
        if let Some(command_name) = maybe_command_name {
            let envs = maybe_prefix.as_ref().map_or_else(
                || HashMap::new(),
                |prefix| {
                    let CmdPrefix(assignments, _) = prefix;
                    assignments
                        .into_iter()
                        .map(|AssignmentWord(key, value)| (key, value))
                        .collect()
                },
            );
            let arguments = maybe_suffix.as_ref().map_or_else(
                || Vec::new(),
                |suffix| {
                    let CmdSuffix(Wordlist(words), _) = suffix;
                    let argument_list: Vec<String> = words
                        .iter()
                        .map(|word| {
                            let Word(argument) = word;
                            argument.clone()
                        })
                        .collect();
                    argument_list
                },
            );

            let result = std::process::Command::new(command_name)
                .args(arguments)
                .envs(envs)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();

            if let Err(e) = result {
                eprintln!("pjsh: {}", e);
                return Err(ExecError);
            }
        }
        Ok(())
    }
}
