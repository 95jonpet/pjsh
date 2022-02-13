use std::{iter::Peekable, str::CharIndices, vec::IntoIter};

use crate::token::{Span, Token, TokenContents};

pub struct Input<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Input<'a> {
    pub fn content(&self, start: usize, end: usize) -> &'a str {
        &self.input[start..end]
    }

    pub fn peek(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }

    pub fn next(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    pub fn len(&self) -> usize {
        self.input.len()
    }
}

pub struct Tokens<'a> {
    it: Peekable<IntoIter<Token<'a>>>,
    eof_token: Token<'a>,
}

impl<'a> Tokens<'a> {
    pub fn len(&self) -> usize {
        self.it.len()
    }

    pub fn peek(&mut self) -> &Token<'a> {
        self.it.peek().unwrap_or(&self.eof_token)
    }

    pub fn next(&mut self) -> Token<'a> {
        self.it.next().unwrap_or_else(|| self.eof_token.clone())
    }

    pub fn next_if(&mut self, func: impl FnOnce(&Token) -> bool) -> Option<Token<'a>> {
        self.it.next_if(func)
    }

    pub fn next_if_eq(&mut self, contents: TokenContents) -> Option<Token> {
        self.next_if(|token| token.it == contents)
    }
}

impl<'a> From<Vec<Token<'a>>> for Tokens<'a> {
    fn from(tokens: Vec<Token<'a>>) -> Self {
        let eof_position = tokens.last().map(|token| token.span.end).unwrap_or(0);
        let eof_token = Token::new(
            Span {
                start: eof_position,
                end: eof_position,
            },
            TokenContents::Eof,
        );
        let it = tokens.into_iter().peekable();
        Self { it, eof_token }
    }
}
