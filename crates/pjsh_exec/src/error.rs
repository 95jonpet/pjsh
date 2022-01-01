use std::fmt::Display;

#[derive(Debug)]
pub enum ExecError {
    ChildSpawnFailed(String),
    Message(String),
    UnknownFileDescriptor(String),
    UnknownProgram(String),
}

impl Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::ChildSpawnFailed(msg) => write!(f, "failed to spawn child process: {}", msg),
            ExecError::Message(message) => write!(f, "{}", message),
            ExecError::UnknownFileDescriptor(fd) => write!(f, "unknown file descriptor: {}", fd),
            ExecError::UnknownProgram(program) => write!(f, "unknown program: {}", program),
        }
    }
}
