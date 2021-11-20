use std::process::ChildStdout;

pub enum Input {
    Piped(ChildStdout),
    Value(String),
}
