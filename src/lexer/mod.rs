mod posix;

use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::cursor::{Cursor, PS2};
use crate::options::Options;
use crate::token::Token;

use self::posix::PosixLexer;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// Default mode of operation.
    Unquoted,
    /// Surrounded by single quotes.
    InSingleQuotes,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Unquoted => write!(f, "unquoted"),
            Mode::InSingleQuotes => write!(f, "single-quoted"),
        }
    }
}

/// An interface for dealing with lexers.
pub trait Lex {
    /// Returns the next [`Token`] from the [`Cursor`].
    /// A special [`Mode`] of operaction can be used to control the behavior.
    fn next_token(&mut self, mode: Mode) -> Token;

    /// Requests the next line of input.
    fn advance_line(&mut self);
}

/// Converts a [`char`] stream from a [`Cursor`] into a [`Token`] stream.
pub struct Lexer {
    cursor: Rc<RefCell<Cursor>>,
    options: Rc<RefCell<Options>>,
    posix_lexer: PosixLexer,
}

impl Lexer {
    pub fn new(cursor: Rc<RefCell<Cursor>>, options: Rc<RefCell<Options>>) -> Self {
        Self {
            cursor,
            options,
            posix_lexer: PosixLexer::new(),
        }
    }
}

impl Lex for Lexer {
    fn next_token(&mut self, mode: Mode) -> Token {
        let token = self.posix_lexer.next_token(&mut self.cursor.borrow_mut());

        if self.options.borrow().debug_lexing {
            eprintln!("[pjsh::lexer] {} mode: {}", mode, token);
        }

        token
    }

    fn advance_line(&mut self) {
        self.cursor.borrow_mut().advance_line(PS2)
    }
}
