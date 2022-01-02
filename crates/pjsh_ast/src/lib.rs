mod command;
mod io;
mod program;

use std::collections::VecDeque;

pub use command::Command;
pub use io::{FileDescriptor, Redirect, RedirectOperator};
pub use program::Program;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    AndOr(AndOr),
    Assignment(Assignment),
    Function(Function),
    Subshell(Program),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub key: Word,
    pub value: Word,
}

impl Assignment {
    pub fn new(key: Word, value: Word) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub args: VecDeque<String>,
    pub body: Program,
}

impl Function {
    pub fn new(name: String, args: VecDeque<String>, body: Program) -> Self {
        Self { name, args, body }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word {
    Literal(String),
    Quoted(String),
    Variable(String),
    Subshell(Program),
    Interpolation(Vec<InterpolationUnit>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolationUnit {
    Literal(String),
    Unicode(char),
    Variable(String),
    Subshell(Program),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndOr {
    pub operators: Vec<AndOrOp>,
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndOrOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline {
    pub is_async: bool,
    pub segments: Vec<PipelineSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineSegment {
    pub command: Command,
}
