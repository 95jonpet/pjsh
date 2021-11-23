use std::{iter::Peekable, vec::IntoIter};

use crate::{
    lex::lexer::{Span, Token},
    tokens::TokenContents,
};

/// A cursor for traversing through a peekable [`Token`] iterator while skipping trivial tokens.
#[derive(Clone)]
pub struct TokenCursor<'a> {
    /// Tokens that the cursor traverses.
    tokens: Peekable<IntoIter<Token<'a>>>,

    /// The token representing the cursor's EOF.
    /// This token is returned upon, and after, consuming all tokens.
    eof_token: Token<'a>,
}

impl<'a> TokenCursor<'a> {
    /// Constructs a new cursor for a predefined set of tokens.
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            eof_token: Token::new(TokenContents::Eof, Span::new(0, 0)),
            tokens: tokens.into_iter().peekable(),
        }
    }

    /// Returns a reference to the next non-trivial [`Token`] while advancing the cursor past
    /// trivial tokens.
    pub fn peek(&mut self) -> &Token<'a> {
        self.skip_trivial_tokens();
        self.tokens.peek().unwrap_or(&self.eof_token)
    }

    /// Returns the next non-trivial [`Token`] while advancing the cursor.
    pub fn next(&mut self) -> Token<'a> {
        self.skip_trivial_tokens();
        self.tokens.next().unwrap_or_else(|| self.eof_token.clone())
    }

    /// Skips all trivial tokens, stopping before the next non-trivial token.
    fn skip_trivial_tokens(&mut self) {
        while is_trivial(self.tokens.peek().unwrap_or(&self.eof_token)) {
            self.tokens.next();
        }
    }
}

/// Returns `true` if a [`Token`] is considered trivial.
/// Trivial tokens are typically discarded.
fn is_trivial(token: &Token) -> bool {
    matches!(
        token.contents,
        TokenContents::Comment | TokenContents::Whitespace
    )
}
