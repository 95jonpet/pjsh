use std::fmt::Display;

use crate::Word;

/// A value pipeline filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    // Word filters.
    /// Transform all letters to lowercase.
    Lower,
    /// Transform all letters to uppercase.
    Upper,

    // List filters.
    /// Return the element with the given index.
    Index(Word),

    /// Return the input list length.
    Len,

    /// Reverse the input list.
    Reverse,

    /// Sort the input list.
    Sort,

    /// Return all unique elements in the input list.
    Unique,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Index(_) => write!(f, "index"),
            Filter::Len => write!(f, "len"),
            Filter::Lower => write!(f, "lower"),
            Filter::Upper => write!(f, "upper"),
            Filter::Reverse => write!(f, "reverse"),
            Filter::Sort => write!(f, "sort"),
            Filter::Unique => write!(f, "unique"),
        }
    }
}
