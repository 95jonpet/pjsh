mod error;
mod lex;
mod parse;
mod token;

pub use error::ParseError;
pub use lex::{
    input::is_whitespace,
    input::Span,
    lexer::{lex, lex_interpolation},
};
pub use parse::{parse, parse_interpolation, ParseResult};
