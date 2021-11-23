mod command;

pub use command::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement<'a> {
    AndOr(AndOr<'a>),
    Assignment(Assignment<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment<'a> {
    pub key: Word<'a>,
    pub value: Word<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word<'a> {
    Literal(&'a str),
    Quoted(String),
    Variable(&'a str),
    Interpolation(Vec<InterpolationUnit<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolationUnit<'a> {
    Literal(&'a str),
    Unicode(char),
    Variable(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct AndOr<'a> {
    pub operators: Vec<AndOrOp>,
    pub pipelines: Vec<Pipeline<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AndOrOp {
    And,
    Or,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pipeline<'a> {
    pub is_async: bool,
    pub segments: Vec<PipelineSegment<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PipelineSegment<'a> {
    pub command: Command<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Redirect {
    pub source: FileDescriptor,
    pub operator: RedirectOperator,
    pub target: FileDescriptor,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectOperator {
    Read,
    Write,
    Append,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileDescriptor {
    Number(usize),
    File(String),
}
