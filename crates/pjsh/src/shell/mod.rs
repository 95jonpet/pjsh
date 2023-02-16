use std::{io, sync::Arc};

use parking_lot::Mutex;
use pjsh_core::Context;
use pjsh_eval::EvalError;
use pjsh_parse::ParseError;

mod command_shell;
pub(crate) mod context;
mod file_shell;
mod interactive_shell;
mod stdin_shell;
pub(crate) mod utils;

pub(crate) use command_shell::CommandShell;
pub(crate) use file_shell::{FileParseShell, FileShell};
pub(crate) use interactive_shell::InteractiveShell;
pub(crate) use stdin_shell::StdinShell;

/// Shell-related error types.
pub enum ShellError {
    /// A generic error with a message.
    Error(String),

    /// A parse error and the input resulting in the error.
    ParseError(ParseError, String),

    /// An evaluation error.
    EvalError(EvalError),

    /// A generic I/O-related error.
    IoError(io::Error),
}

/// Result type for shell operations.
pub type ShellResult<T> = Result<T, ShellError>;

/// A shell responsible for reading input, parsing, and evaluating it.
pub trait Shell {
    /// Initializes the shell.
    fn init(&mut self) -> ShellResult<()>;

    /// Runs the shell.
    ///
    /// This should read input, parse, and evaluate it.
    fn run(&mut self, context: Arc<Mutex<Context>>) -> ShellResult<()>;

    /// Exits the shell.
    fn exit(self) -> ShellResult<()>;
}
