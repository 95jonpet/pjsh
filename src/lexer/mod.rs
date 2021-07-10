mod double_quoted_mode;
mod single_quoted_mode;
mod unquoted_mode;

use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::cursor::Cursor;
use crate::options::Options;
use crate::token::Token;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// Default mode of operation.
    Unquoted,
    /// Surrounded by single quotes.
    InSingleQuotes,
    /// Surrounded by double quotes.
    InDoubleQuotes,
    // Arithmetic,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Unquoted => write!(f, "unquoted"),
            Mode::InSingleQuotes => write!(f, "single-quoted"),
            Mode::InDoubleQuotes => write!(f, "double-quoted"),
        }
    }
}

pub trait Lex {
    /// Returns the next [`Token`] from the [`Cursor`].
    /// A special [`Mode`] of operaction can be used to control the behavior.
    fn next_token(&mut self, mode: Mode) -> Token;
}

/// Converts a [`char`] stream from a [`Cursor`] into a [`Token`] stream.
pub struct Lexer {
    cursor: Cursor,
    options: Rc<RefCell<Options>>,
}

impl Lexer {
    pub fn new(cursor: Cursor, options: Rc<RefCell<Options>>) -> Self {
        Self { cursor, options }
    }
}

impl Lex for Lexer {
    fn next_token(&mut self, mode: Mode) -> Token {
        let token = match mode {
            Mode::InDoubleQuotes => double_quoted_mode::next_double_quoted_token(&mut self.cursor),
            Mode::InSingleQuotes => single_quoted_mode::next_single_quoted_token(&mut self.cursor),
            Mode::Unquoted => unquoted_mode::next_unquoted_token(&mut self.cursor),
        };

        if self.options.borrow().debug_lexing {
            eprintln!("[pjsh::lexer] {} mode: {}", mode, token);
        }

        token
    }
}
