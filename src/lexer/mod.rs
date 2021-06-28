mod single_quoted_mode;
mod unquoted_mode;

use crate::cursor::Cursor;
use crate::token::Token;

#[derive(Clone, Copy)]
pub enum Mode {
    /// Default mode of operation.
    Unquoted,
    /// Surrounded by single quotes.
    InSingleQuotes,
    // InDoubleQuotes,
    // Arithmetic,
}

/// Converts a [`char`] stream from a [`Cursor`] into a [`Token`] stream.
pub struct Lexer {
    cursor: Cursor,
}

impl Lexer {
    pub fn new(cursor: Cursor) -> Self {
        Self { cursor }
    }

    /// Returns the next [`Token`] from the [`Cursor`].
    /// A special [`Mode`] of operaction can be used to control the behavior.
    pub fn next_token(&mut self, mode: Mode) -> Token {
        match mode {
            Mode::InSingleQuotes => single_quoted_mode::next_single_quoted_token(&mut self.cursor),
            Mode::Unquoted => unquoted_mode::next_unquoted_token(&mut self.cursor),
        }
    }

    fn operator_token(&self, lexeme: &str) -> Option<Token> {
        match lexeme {
            // "{" => Some(Token::LBrace),
            // "}" => Some(Token::RBrace),
            // "!" => Some(Token::Bang),
            // "in" => Some(Token::In),
            _ => None,
        }
    }
}
