mod single_quoted_mode;
mod unquoted_mode;

use crate::cursor::Cursor;
use crate::token::Token;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// Default mode of operation.
    Unquoted,
    /// Surrounded by single quotes.
    InSingleQuotes,
    // InDoubleQuotes,
    // Arithmetic,
}

pub trait Lex {
    /// Returns the next [`Token`] from the [`Cursor`].
    /// A special [`Mode`] of operaction can be used to control the behavior.
    fn next_token(&mut self, mode: Mode) -> Token;
}

/// Converts a [`char`] stream from a [`Cursor`] into a [`Token`] stream.
pub struct Lexer {
    cursor: Cursor,
}

impl Lexer {
    pub fn new(cursor: Cursor) -> Self {
        Self { cursor }
    }
}

impl Lex for Lexer {
    fn next_token(&mut self, mode: Mode) -> Token {
        match mode {
            Mode::InSingleQuotes => single_quoted_mode::next_single_quoted_token(&mut self.cursor),
            Mode::Unquoted => unquoted_mode::next_unquoted_token(&mut self.cursor),
        }
    }
}
