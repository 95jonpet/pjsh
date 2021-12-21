use std::{fmt::Display, process::Child};

pub type Result = std::result::Result<Option<Child>, ExecError>;

#[derive(Debug)]
pub enum ExecError {
    ChildSpawnFailed(String),
    Message(String),
    UnknownProgram(String),
}

impl Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::ChildSpawnFailed(msg) => write!(f, "failed to spawn child process: {}", msg),
            ExecError::Message(message) => write!(f, "{}", message),
            ExecError::UnknownProgram(program) => write!(f, "unknown program: {}", program),
        }
    }
}
