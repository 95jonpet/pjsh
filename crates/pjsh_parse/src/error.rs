use thiserror::Error;

use crate::{
    token::{Token, TokenContents},
    Span,
};

/// Parse errors are returned by a parser when input cannot be parsed.
///
/// Note that some parse errors are recoverable, and that some errors may expected withing certain
/// contexts.
#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    /// Error indicating that a parsed subshell contains no statements.
    #[error("empty subshell")]
    EmptySubshell(Span),

    /// Error indicating that an expected token was not found in the input.
    /// The current sequence of tokens cannot be parsed in this context.
    ///
    /// Note that the token may still be valid in a different context.
    #[error("invalid token (expected `{0:?}`), found {:?}", .1.contents)]
    ExpectedToken(TokenContents, Token), // (expected, found).

    /// Error indicating that there is no more input to parse while parsing a started sequence.
    ///
    /// This error is recoverable, and interactive shells should prompt the user for more input.
    #[error("incomplete sequence")]
    IncompleteSequence,

    /// Error indicating that the syntax is invalid.
    ///
    /// Contains an error message.
    ///
    /// This error is not recoverable.
    #[error("invalid syntax: {0}")]
    InvalidSyntax(String),

    /// Error indicating that there is no more input to parse.
    ///
    /// This error is only returned before consuming tokens in a new sequence.
    /// [`ParseError::IncompleteSequence`] should instead be returned when within a sequence.
    ///
    /// This error could also mean that the input has been fully parsed.
    #[error("unexpected end of file")]
    UnexpectedEof,

    /// Error indicating that an unexpected token was found in the input.
    /// The current sequence of tokens cannot be parsed in this context.
    ///
    /// Note that the token may still be valid in a different context.
    #[error("unexpected token {:?}", .0.contents)]
    UnexpectedToken(Token),
}

impl ParseError {
    /// Returns a help text associated with the error.
    pub fn help(&self) -> &str {
        match self {
            ParseError::EmptySubshell(_) => "this subshell is empty",
            ParseError::ExpectedToken(_, _) => "another token is expected here",
            ParseError::IncompleteSequence => "this sequence is incomplete",
            ParseError::InvalidSyntax(_) => "this syntax is invalid",
            ParseError::UnexpectedEof => "EOF was encountered here",
            ParseError::UnexpectedToken(_) => "this token is unexpected here",
        }
    }

    /// Returns the positional span in which the error resides.
    pub fn span(&self) -> Option<Span> {
        match self {
            ParseError::EmptySubshell(span) => Some(*span),
            ParseError::ExpectedToken(_, found) => Some(found.span),
            ParseError::UnexpectedToken(token) => Some(token.span),
            _ => None,
        }
    }
}
