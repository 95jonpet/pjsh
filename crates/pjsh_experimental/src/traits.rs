use crate::{
    error::{LexError, ParseError},
    input::{Input, Tokens},
};

pub type LexResult<T> = Result<T, LexError>;

pub trait Lex<'a, T: 'a> {
    fn lex(&self, input: &mut Input<'a>) -> LexResult<T>;
}

pub type ParseResult<T> = Result<T, ParseError>;

pub trait Parse<'a, T: 'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<T>;
}
