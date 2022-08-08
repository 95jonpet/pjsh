use std::{path::PathBuf, sync::Arc};

use parking_lot::Mutex;

use crate::{command::Io, Context};

type ExitCode = i32;

/// Represents an action that should be performed by the shell.
///
/// Actions allow commands to perform tasks that the shell is normally
/// responsible for, and that a command itself is unable to perform.
///
/// The shell performs actions asynchronously. Thus, a command cannot directly
/// rely on an action's output, but rather on callbacks.
pub enum Action {
    /// Exit the current scope.
    ExitScope(ExitCode),

    /// Interpolate a string and call a function with the interpolated value as
    /// an argument, or an error message if it cannot be interpolated.
    Interpolate(String, Box<dyn FnOnce(Io, Result<&str, &str>) -> ExitCode>),

    /// Resolve the type of a command and call a function with it as an
    /// argument.
    ResolveCommandType(String, Box<dyn FnOnce(Io, CommandType) -> ExitCode>),

    /// Resolve the path to a command and call a function with it as an
    /// argument.
    ResolveCommandPath(
        String,
        Box<dyn FnOnce(String, Io, Option<&PathBuf>) -> ExitCode>,
    ),

    /// Source a file within a context with some additional arguments. The first
    /// argument should correspond with the name of the sourced file.
    SourceFile(PathBuf, Arc<Mutex<Context>>, Vec<String>),
}

/// Command types.
pub enum CommandType {
    /// User-defined alias.
    Alias(String),
    /// Shell built-in.
    Builtin,
    /// User-defined function.
    Function,
    /// A local program.
    Program(PathBuf),
    /// Unknown command - the command cannot be resolved.
    Unknown,
}
