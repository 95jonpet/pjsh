use super::Action;

/// Represents the result of executing a command.
pub struct CommandResult {
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

impl CommandResult {
    /// Constructs a new `CommandResult` without any actions.
    pub fn code(code: i32) -> Self {
        Self {
            code,
            actions: Vec::new(),
        }
    }

    /// Constructs a new `CommandResult` with a code and some actions.
    pub fn with_actions(code: i32, actions: Vec<Action>) -> Self {
        Self { code, actions }
    }
}

impl From<i32> for CommandResult {
    fn from(code: i32) -> Self {
        Self::code(code)
    }
}

impl From<Vec<Action>> for CommandResult {
    fn from(actions: Vec<Action>) -> Self {
        Self::with_actions(0, actions)
    }
}
