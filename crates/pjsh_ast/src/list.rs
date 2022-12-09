use std::fmt::{Debug, Display};

use crate::Word;

/// Represents a list of words.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct List {
    /// Items in the list.
    pub(crate) items: Vec<Word>,
}

impl List {
    /// Appends a word to the back of the list.
    pub fn push(&mut self, word: Word) {
        self.items.push(word);
    }
}

impl From<Vec<Word>> for List {
    fn from(items: Vec<Word>) -> Self {
        Self { items }
    }
}

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.items.fmt(f)
    }
}
