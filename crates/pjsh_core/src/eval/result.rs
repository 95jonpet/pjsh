use std::{fmt::Display, process::Child};

pub type Result = std::result::Result<Value, ExecError>;

pub enum Value {
    Child(Child),
    Empty,
    String(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Child(child) => write!(f, "process {}", child.id()),
            Value::Empty => Ok(()),
            Value::String(string) => write!(f, "{}", string),
        }
    }
}

pub enum ExecError {
    ChildSpawnFailed,
    Message(String),
    UnknownProgram(String),
    Value(Value),
}

impl Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::ChildSpawnFailed => write!(f, "failed to spawn child process"),
            ExecError::Message(message) => write!(f, "{}", message),
            ExecError::UnknownProgram(program) => write!(f, "unknown program: {}", program),
            ExecError::Value(value) => value.fmt(f),
        }
    }
}

impl From<Child> for Value {
    fn from(it: Child) -> Self {
        Value::Child(it)
    }
}

impl From<String> for Value {
    fn from(it: String) -> Self {
        Value::String(it)
    }
}
