use std::fmt::Display;

use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnconsumedToken(Token),
    UnexpectedToken(Token),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnconsumedToken(token) => write!(f, "unconsumed token {}", token),
            Self::UnexpectedToken(token) => write!(f, "unexpected token {}", token),
        }
    }
}
