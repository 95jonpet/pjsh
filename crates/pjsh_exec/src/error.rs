use std::{fmt::Display, path::PathBuf};

use pjsh_core::utils::path_to_string;

#[derive(Debug)]
pub enum ExecError {
    ChildSpawnFailed(String),
    InvalidProgramPath(PathBuf),
    Message(String),
    MissingFunctionArgument(String),
    UnknownFileDescriptor(String),
    UnknownProgram(String),
}

impl Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::ChildSpawnFailed(msg) => write!(f, "failed to spawn child process: {}", msg),
            ExecError::InvalidProgramPath(path) => {
                write!(f, "invalid program path: {}", path_to_string(path))
            }
            ExecError::Message(message) => write!(f, "{}", message),
            ExecError::MissingFunctionArgument(arg) => {
                write!(f, "missing function argument: {}", arg)
            }
            ExecError::UnknownFileDescriptor(fd) => write!(f, "unknown file descriptor: {}", fd),
            ExecError::UnknownProgram(program) => write!(f, "unknown program: {}", program),
        }
    }
}
