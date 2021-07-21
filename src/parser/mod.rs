mod adapter;
mod command;
mod error;
mod io;
mod meta;
mod pipeline;
pub(crate) mod posix;
mod word;

use self::{adapter::LexerAdapter, error::ParseError};

/// An interface for dealing with parsers.
pub trait Parse {
    /// The type of elements being parsed.
    type Item;

    /// Parses and returns the next [`Item`] using a [`LexerAdapter`].
    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError>;
}
