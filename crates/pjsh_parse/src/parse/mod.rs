use std::collections::HashMap;

use pjsh_ast::{Program, Word};

use crate::{lex::lexer::LexError, ParseError};

use self::{cursor::TokenCursor, program::parse_program, word::parse_word};

mod command;
mod condition;
mod cursor;
mod filter;
mod iterable;
mod pipeline;
mod program;
mod statement;
mod utils;
mod word;

/// A specialized [`Result`] type for parsing.
pub type ParseResult<T> = Result<T, ParseError>;

/// Parses a [`Program`] by consuming some input `src` in its entirety.
///
/// # Errors
///
/// This function will return an error if a program can't be parsed.
pub fn parse(src: &str, aliases: &HashMap<String, String>) -> ParseResult<Program> {
    match crate::lex(src, aliases) {
        Ok(tokens) => parse_program(&mut TokenCursor::from(tokens)),
        Err(LexError::UnexpectedEof) => Err(ParseError::UnexpectedEof),
        Err(error) => Err(ParseError::InvalidSyntax(error.to_string())),
    }
}

/// Parses a [`Word`] from within an interpolation.
///
/// # Errors
///
/// This function will return an error if a word can't be parsed.
pub fn parse_interpolation(src: &str) -> ParseResult<Word> {
    match crate::lex_interpolation(src) {
        Ok(token) => parse_word(&mut TokenCursor::from(vec![token])),
        Err(LexError::UnexpectedEof) => Err(ParseError::UnexpectedEof),
        Err(error) => Err(ParseError::InvalidSyntax(error.to_string())),
    }
}
