use std::fmt::Display;

pub enum ExecError {
    /// No command specified.
    MissingCommand,
    /// The given command cannot be resolved.
    UnknownCommand(String),
}

impl Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCommand => write!(f, "no command specified"),
            Self::UnknownCommand(command) => write!(f, "unknown command `{}`", command),
        }
    }
}
