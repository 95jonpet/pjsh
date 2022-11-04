use std::process::Command;

use super::Action;

/// Represents the result of executing a command.
pub enum CommandResult {
    Builtin(BuiltinCommandResult),
    Process(ProcessCommandResult),
}

pub struct BuiltinCommandResult {
    /// Exit code.
    ///
    /// Successful execution is typically represented by `0`.
    pub code: i32,

    /// Actions that should be taken by the shell after executing the command.
    ///
    /// Actions allow commands to perform tasks that the shell is normally
    /// responsible for, and that a command itself is unable to perform.
    pub actions: Vec<Action>,
}

pub struct ProcessCommandResult {
    pub command: Command,
}

impl CommandResult {
    /// Constructs a new `CommandResult` without any actions.
    pub fn code(code: i32) -> Self {
        Self::Builtin(BuiltinCommandResult {
            code,
            actions: Vec::new(),
        })
    }

    /// Constructs a new `CommandResult` with a code and some actions.
    pub fn with_actions(code: i32, actions: Vec<Action>) -> Self {
        Self::Builtin(BuiltinCommandResult { code, actions })
    }
}

impl From<Command> for CommandResult {
    fn from(command: Command) -> Self {
        Self::Process(ProcessCommandResult { command })
    }
}
