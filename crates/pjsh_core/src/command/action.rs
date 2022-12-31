use std::path::PathBuf;

use crate::command::Io;

type ExitCode = i32;

type InterpolateCallback = dyn Fn(Io, Result<String, String>) -> ExitCode;

type ResolveCommandPathCallback = dyn Fn(String, Io, Option<&PathBuf>) -> ExitCode;

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
    Interpolate(String, Box<InterpolateCallback>),

    /// Resolve the type of a command and call a function with it as an
    /// argument.
    ResolveCommandType(String, Box<dyn Fn(Io, String, CommandType) -> ExitCode>),

    /// Resolve the path to a command and call a function with it as an
    /// argument.
    ResolveCommandPath(String, Box<ResolveCommandPathCallback>),
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
