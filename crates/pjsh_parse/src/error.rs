use std::fmt::Display;

use crate::{token::Token, Span};

/// Parse errors are returned by a parser when input cannot be parsed.
///
/// Note that some parse errors are recoverable, and that some errors may expected withing certain
/// contexts.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Error indicating that a parsed subshell contains no statements.
    EmptySubshell,

    /// Error indicating that there is no more input to parse while parsing a started sequence.
    ///
    /// This error is recoverable, and interactive shells should prompt the user for more input.
    IncompleteSequence,

    /// Error indicating that the syntax is invalid.
    ///
    /// This error is not recoverable.
    InvalidSyntax(String),

    /// Error indicating that there is no more input to parse.
    ///
    /// This error is only returned before consuming tokens in a new sequence.
    /// [`ParseError::IncompleteSequence`] should instead be returned when within a sequence.
    ///
    /// This error could also mean that the input has been fully parsed.
    UnexpectedEof,

    /// Error indicating that an unexpected token was found in the input.
    /// The current sequence of tokens cannot be parsed in this context.
    ///
    /// Note that the token may still be valid in a different context.
    UnexpectedToken(Token),
}

impl ParseError {
    /// Returns a help text associated with the error.
    pub fn help(&self) -> &str {
        match self {
            ParseError::EmptySubshell => "this subshell is empty",
            ParseError::IncompleteSequence => "this sequence is incomplete",
            ParseError::InvalidSyntax(_) => "this syntax is invalid",
            ParseError::UnexpectedEof => "EOF was encountered here",
            ParseError::UnexpectedToken(_) => "this token is unexpected here",
        }
    }

    /// Returns the positional span in which the error resides.
    pub fn span(&self) -> Option<Span> {
        match self {
            ParseError::UnexpectedToken(token) => Some(token.span),
            _ => None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptySubshell => write!(f, "empty subshell"),
            ParseError::IncompleteSequence => write!(f, "incomplete sequence"),
            ParseError::InvalidSyntax(message) => write!(f, "invalid syntax: {message}"),
            ParseError::UnexpectedEof => write!(f, "unexpected end of file"),
            ParseError::UnexpectedToken(token) => {
                write!(f, "unexpected token: {:?}", token.contents)
            }
        }
    }
}
