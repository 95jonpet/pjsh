use std::fmt::Display;

use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedCharSequence,
    UnexpectedToken(Token),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCharSequence => write!(f, "unexpected character sequence"),
            Self::UnexpectedToken(token) => write!(f, "unexpected token {:?}", token),
        }
    }
}
