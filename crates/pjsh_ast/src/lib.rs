mod command;
mod io;
mod program;

pub use command::Command;
pub use io::{FileDescriptor, Redirect, RedirectOperator};
pub use program::Program;

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    AndOr(AndOr),
    Assignment(Assignment),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    pub key: Word,
    pub value: Word,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word {
    Literal(String),
    Quoted(String),
    Variable(String),
    Interpolation(Vec<InterpolationUnit>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolationUnit {
    Literal(String),
    Unicode(char),
    Variable(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct AndOr {
    pub operators: Vec<AndOrOp>,
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AndOrOp {
    And,
    Or,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pipeline {
    pub is_async: bool,
    pub segments: Vec<PipelineSegment>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PipelineSegment {
    pub command: Command,
}
