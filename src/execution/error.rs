use std::fmt::Display;

/// Represents an error during execution.
pub enum ExecError {
    /// Illegal pipeline definition.
    MalformedPipeline,
    /// No command specified.
    MissingCommand,
    /// The given command cannot be resolved.
    UnknownCommand(String),
    /// The required parameter is null or not set. Optional message.
    ParameterNullOrNotSet(String, Option<String>),
}

impl Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedPipeline => write!(f, "malformed pipeline"),
            Self::MissingCommand => write!(f, "no command specified"),
            Self::ParameterNullOrNotSet(name, Some(message)) => write!(f, "{}: {}", name, message),
            Self::ParameterNullOrNotSet(name, None) => {
                write!(f, "{}: parameter null or not set", name)
            }
            Self::UnknownCommand(command) => write!(f, "unknown command `{}`", command),
        }
    }
}
