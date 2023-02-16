use std::{collections::HashMap, io::stdin, sync::Arc};

use crate::Shell;

use super::{
    utils::{eval_program, exit_on_error},
    ShellError, ShellResult,
};
use parking_lot::Mutex;
use pjsh_core::Context;
use pjsh_parse::{parse, ParseError};

/// A non-interactive shell that reads input from stdin.
pub struct StdinShell;

impl Shell for StdinShell {
    fn init(&mut self) -> ShellResult<()> {
        Ok(()) // Intentionally left blank.
    }

    fn run(&mut self, context: Arc<Mutex<Context>>) -> ShellResult<()> {
        let aliases = HashMap::new();

        loop {
            let mut line = String::new();
            match stdin().read_line(&mut line) {
                Ok(0) => break, // No more input to read.
                Ok(_) => (),
                Err(_) => break, // No more input to read.
            }

            // Repeatedly ask for lines of input until a valid program can be executed.
            loop {
                match parse(&line, &aliases) {
                    // If a valid program can be parsed from the buffer, execute it.
                    Ok(program) => {
                        eval_program(&program, &mut context.lock(), exit_on_error)?;
                        break;
                    }

                    // If more input is required, prompt for more input and loop again.
                    // The next line of input will be appended to the buffer and parsed.
                    Err(ParseError::IncompleteSequence | ParseError::UnexpectedEof) => {
                        match stdin().read_line(&mut line) {
                            Ok(0) => {
                                return Err(ShellError::ParseError(ParseError::UnexpectedEof, line))
                            }
                            Ok(_) => continue,
                            Err(error) => return Err(ShellError::IoError(error)),
                        }
                    }

                    // Unrecoverable error.
                    Err(error) => {
                        return Err(ShellError::ParseError(error, line));
                    }
                }
            }
        }

        Ok(())
    }

    fn exit(self) -> ShellResult<()> {
        Ok(()) // Intentionally left blank.
    }
}
