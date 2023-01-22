use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use pjsh_core::Context;
use pjsh_parse::parse;

use crate::Shell;

use super::{
    utils::{eval_program, exit_on_error},
    ShellError, ShellResult,
};

/// A shell that executes a command from a string.
pub struct CommandShell {
    /// Command to execute.
    command: String,
}

impl CommandShell {
    /// Constructs a new file shell.
    pub fn new(command: String) -> Self {
        Self { command }
    }
}

impl Shell for CommandShell {
    fn init(&mut self) -> ShellResult<()> {
        Ok(()) // Intentionally left blank.
    }

    fn run(&mut self, context: Arc<Mutex<Context>>) -> ShellResult<()> {
        // Non-interactive shells should not use aliases.
        let aliases = &HashMap::new();

        let program =
            parse(&self.command, aliases).map_err(|error| ShellError::ParseError(error, None))?;
        eval_program(&program, &mut context.lock(), exit_on_error)
    }

    fn exit(self) -> ShellResult<()> {
        Ok(()) // Intentionally left blank.
    }
}
