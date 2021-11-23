use crate::Word;

#[derive(Debug, PartialEq, Eq)]
pub struct Redirect<'a> {
    pub source: FileDescriptor<'a>,
    pub target: FileDescriptor<'a>,
    pub operator: RedirectOperator,
}

impl<'a> Redirect<'a> {
    pub fn new(
        source: FileDescriptor<'a>,
        operator: RedirectOperator,
        target: FileDescriptor<'a>,
    ) -> Self {
        Self {
            source,
            operator,
            target,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectOperator {
    Write,
    Append,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileDescriptor<'a> {
    Number(usize),
    File(Word<'a>),
}
