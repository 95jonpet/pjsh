use std::{collections::HashMap, fs::File, io::Read, sync::Arc};

use parking_lot::Mutex;
use pjsh_core::Context;
use pjsh_parse::parse;

use crate::Shell;

use super::{
    utils::{eval_program, exit_on_error},
    ShellError, ShellResult,
};

/// A shell that executes a script file.
pub struct FileShell {
    /// Script file to execute.
    file: File,
}

impl FileShell {
    /// Constructs a new file shell.
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl Shell for FileShell {
    fn init(&mut self) -> ShellResult<()> {
        Ok(()) // Intentionally left blank.
    }

    fn run(&mut self, context: Arc<Mutex<Context>>) -> ShellResult<()> {
        // Non-interactive shells should not use aliases.
        let aliases = &HashMap::new();

        let mut src = String::new();
        self.file
            .read_to_string(&mut src)
            .map_err(ShellError::IoError)?;

        let program = parse(&src, aliases).map_err(|error| ShellError::ParseError(error, None))?;
        eval_program(&program, &mut context.lock(), exit_on_error)
    }

    fn exit(self) -> ShellResult<()> {
        Ok(()) // Intentionally left blank.
    }
}

/// A shell that parses a script file.
pub struct FileParseShell {
    /// Script file to parse.
    file: File,
}

impl FileParseShell {
    /// Constructs a new file shell.
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl Shell for FileParseShell {
    fn init(&mut self) -> ShellResult<()> {
        Ok(()) // Intentionally left blank.
    }

    fn run(&mut self, _context: Arc<Mutex<Context>>) -> ShellResult<()> {
        // Non-interactive shells should not use aliases.
        let aliases = &HashMap::new();

        let mut src = String::new();
        self.file
            .read_to_string(&mut src)
            .map_err(ShellError::IoError)?;

        let program = parse(&src, aliases).map_err(|error| ShellError::ParseError(error, None))?;
        println!("{:#?}", program);

        Ok(())
    }

    fn exit(self) -> ShellResult<()> {
        Ok(()) // Intentionally left blank.
    }
}
