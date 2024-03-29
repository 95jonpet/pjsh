use std::fmt::Display;

use pjsh_core::{FileDescriptorError, FilterError};

pub type EvalResult<T> = Result<T, EvalError>;

#[derive(Debug)]
pub enum EvalError {
    FileDescriptorError(usize, FileDescriptorError),
    FilterError(String, FilterError),
    ChildSpawnFailed(std::io::Error),
    ContextCloneFailed(std::io::Error),
    CreatePipeFailed(std::io::Error),
    InvalidIndex,
    InvalidListInterpolation(String),
    InvalidRegex(String),         // Contains an error message.
    InvalidValuePipeline(String), // Contains an error message.
    InvalidVariableType {
        variable: String,
        expected_type: String,
        actual_type: String,
    },
    IoError(std::io::Error), // General IO catch-all error.
    PipelineFailed(Vec<std::io::Error>),
    UnboundFunctionArguments(Vec<String>),
    UndefinedFileDescriptor(usize),
    UndefinedFunctionArguments(Vec<String>),
    UndefinedVariable(String),
    UnknownCommand(String),
    UnknownFilter(String),
}

impl Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::FileDescriptorError(fd, err) => match err {
                FileDescriptorError::UnusableForOutput => {
                    write!(f, "file descriptor {fd} cannot be used for output")
                }
                FileDescriptorError::UnusableForInput => {
                    write!(f, "file descriptor {fd} cannot be used for input")
                }
                FileDescriptorError::FileNotReadable(path, err) => {
                    write!(f, "file '{}' is not readable: {err}", path.display())
                }
                FileDescriptorError::FileNotWritable(path, err) => {
                    write!(f, "file '{}' is not writable: {err}", path.display())
                }
            },
            EvalError::FilterError(filter, error) => write!(f, "{filter}: {error}"),
            EvalError::ChildSpawnFailed(err) => write!(f, "failed to spawn child process: {err}"),
            EvalError::ContextCloneFailed(err) => write!(f, "failed to clone context: {err}"),
            EvalError::CreatePipeFailed(err) => write!(f, "failed to create pipe: {err}"),
            EvalError::InvalidIndex => write!(f, "invalid index"),
            EvalError::InvalidListInterpolation(var) => {
                write!(f, "invalid list interpolation: {var}")
            }
            EvalError::InvalidRegex(msg) => write!(f, "invalid regex: {msg}"),
            EvalError::InvalidValuePipeline(msg) => write!(f, "invalid value pipeline: {msg}"),
            EvalError::InvalidVariableType {
                variable,
                expected_type,
                actual_type,
            } => write!(
                f,
                "{variable}: invalid type (expected {expected_type}), found {actual_type}"
            ),
            EvalError::IoError(err) => write!(f, "input/output error: {err}"),
            EvalError::PipelineFailed(errors) => write!(f, "pipeline failed: {:?}", errors),
            EvalError::UnboundFunctionArguments(args) => {
                write!(f, "unbound function arguments: {}", args.join(", "))
            }
            EvalError::UndefinedFileDescriptor(fd) => write!(f, "undefined file descriptor: {fd}"),
            EvalError::UndefinedFunctionArguments(args) => {
                write!(f, "undefined function arguments: {}", args.join(", "))
            }
            EvalError::UndefinedVariable(variable) => write!(f, "undefined variable: {variable}"),
            EvalError::UnknownCommand(command) => write!(f, "unknown command: {command}"),
            EvalError::UnknownFilter(filter) => write!(f, "unknown filter: {filter}"),
        }
    }
}
