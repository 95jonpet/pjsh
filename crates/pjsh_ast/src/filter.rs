use crate::Word;

/// A value pipeline filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    /// Filter name.
    pub name: Word,

    /// Filter arguments.
    pub args: Vec<Word>,
}
