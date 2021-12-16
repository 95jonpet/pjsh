use std::{iter::Peekable, vec::IntoIter};

use crate::{
    lex::lexer::{Span, Token},
    tokens::TokenContents,
};

/// A cursor for traversing through a peekable [`Token`] iterator while skipping trivial tokens.
#[derive(Clone)]
pub struct TokenCursor {
    /// Tokens that the cursor traverses.
    tokens: Peekable<IntoIter<Token>>,

    /// The token representing the cursor's EOF.
    /// This token is returned upon, and after, consuming all tokens.
    eof_token: Token,
}

impl TokenCursor {
    /// Constructs a new cursor for a predefined set of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            eof_token: Token::new(TokenContents::Eof, Span::new(0, 0)),
            tokens: tokens.into_iter().peekable(),
        }
    }

    /// Returns a reference to the next non-trivial [`Token`] while advancing the cursor past
    /// trivial tokens.
    pub fn peek(&mut self) -> &Token {
        self.skip_trivial_tokens();
        self.tokens.peek().unwrap_or(&self.eof_token)
    }

    /// Returns the next non-trivial [`Token`] while advancing the cursor.
    pub fn next(&mut self) -> Token {
        self.skip_trivial_tokens();
        self.tokens.next().unwrap_or_else(|| self.eof_token.clone())
    }

    /// Consume and return the next token if a condition is true.
    ///
    /// If `func` returns `true` for the next token, consume and return it.
    /// Otherwise, return `None`.
    ///
    /// Skips trival tokens before evaluating the condition.
    pub fn next_if(&mut self, func: impl FnOnce(&Token) -> bool) -> Option<Token> {
        self.skip_trivial_tokens();
        self.tokens.next_if(func)
    }

    /// Consume and return the next token if `contents` match the next token's contents.
    /// Otherwise, return `None`.
    ///
    /// Skips trival tokens.
    pub fn next_if_eq(&mut self, contents: TokenContents) -> Option<Token> {
        self.skip_trivial_tokens();
        self.next_if(|token| token.contents == contents)
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
