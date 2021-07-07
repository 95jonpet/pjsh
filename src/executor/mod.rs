mod error;
mod exit_status;

use std::{collections::HashMap, process::Stdio};

use crate::ast::{
    AndOr, AndOrPart, AssignmentWord, CmdPrefix, CmdSuffix, Command, CompleteCommand,
    CompleteCommands, List, ListPart, PipeSequence, Pipeline, Program, SeparatorOp, SimpleCommand,
    Word, Wordlist,
};

use self::{error::ExecError, exit_status::ExitStatus};

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self {}
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
        separator_op: &SeparatorOp,
    ) -> Result<ExitStatus, ExecError> {
        let AndOr(parts) = and_or;
        let mut part_iterator = parts.into_iter();
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

            match result {
                Ok(status) => Ok(ExitStatus::new(status.code().unwrap())),
                Err(_) => Err(ExecError::UnknownCommand(command_name.to_string())),
            }
        } else {
            Err(ExecError::MissingCommand)
        }
    }
}
