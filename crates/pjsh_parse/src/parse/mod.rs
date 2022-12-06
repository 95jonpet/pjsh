use std::collections::HashMap;

use pjsh_ast::{Program, Word};

use crate::{lex::lexer::LexError, ParseError};

use self::{cursor::TokenCursor, program::parse_program, word::parse_word};

mod command;
mod condition;
mod cursor;
mod iterable;
mod pipeline;
mod program;
mod statement;
mod utils;
mod word;

pub type ParseResult<T> = Result<T, ParseError>;

/// Tries to parse a [`Program`] by consuming some input `src` in its entirety.
/// A [`ParseError`] is returned if a program can't be parsed.
pub fn parse(src: &str, aliases: &HashMap<String, String>) -> Result<Program, ParseError> {
    match crate::lex(src, aliases) {
        Ok(tokens) => parse_program(&mut TokenCursor::from(tokens)),
        Err(LexError::UnexpectedEof) => Err(ParseError::UnexpectedEof),
        Err(error) => Err(ParseError::InvalidSyntax(error.to_string())),
    }
}

/// Tries to parse a [`Word`] from within an interpolation.
/// A [`ParseError`] is returned if a program can't be parsed.
pub fn parse_interpolation(src: &str) -> Result<Word, ParseError> {
    match crate::lex_interpolation(src) {
        Ok(token) => parse_word(&mut TokenCursor::from(vec![token])),
        Err(LexError::UnexpectedEof) => Err(ParseError::UnexpectedEof),
        Err(error) => Err(ParseError::InvalidSyntax(error.to_string())),
    }
}
