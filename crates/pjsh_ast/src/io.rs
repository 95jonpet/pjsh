use crate::Word;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    pub source: FileDescriptor,
    pub target: FileDescriptor,
    pub operator: RedirectOperator,
}

impl Redirect {
    pub fn new(source: FileDescriptor, operator: RedirectOperator, target: FileDescriptor) -> Self {
        Self {
            source,
            operator,
            target,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectOperator {
    Write,
    Append,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileDescriptor {
    Number(usize),
    File(Word),
}
