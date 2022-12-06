use std::{iter::Peekable, vec::IntoIter};

use crate::{
    token::{Token, TokenContents},
    Span,
};

/// The newline mode determines how a [`TokenCursor`] handles newline tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
enum NewlineMode {
    /// Newline is treated as newline.
    Newline,
    /// Newline is replaced by whitespace.
    Whitespace,
}

/// A cursor for traversing through a peekable [`Token`] iterator while skipping trivial tokens.
#[derive(Clone, Debug)]
pub struct TokenCursor {
    /// Tokens that the cursor traverses.
    tokens: Peekable<IntoIter<Token>>,

    /// The token representing the cursor's EOF.
    ///
    /// This token is returned upon, and after, consuming all tokens.
    eof_token: Token,

    /// Mode of operation for newline tokens.
    newline_mode: NewlineMode,
}

impl TokenCursor {
    /// Returns a reference to the next non-trivial [`Token`] while advancingblank the cursor past
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
    /// Skips trivial tokens before evaluating the condition.
    pub fn next_if(&mut self, func: impl FnOnce(&Token) -> bool) -> Option<Token> {
        self.skip_trivial_tokens();
        self.tokens.next_if(func)
    }

    /// Consume and return the next token if `contents` match the next token's contents.
    /// Otherwise, return `None`.
    ///
    /// Skips trivial tokens.
    pub fn next_if_eq(&mut self, contents: TokenContents) -> Option<Token> {
        self.skip_trivial_tokens();
        self.next_if(|token| token.contents == contents)
    }

    /// Further operations should treat newline as whitespace if `is_whitespace` is `true`.
    pub fn newline_is_whitespace(&mut self, is_whitespace: bool) {
        self.newline_mode = match is_whitespace {
            true => NewlineMode::Whitespace,
            false => NewlineMode::Newline,
        };
    }

    /// Skips all trivial tokens, stopping before the next non-trivial token.
    fn skip_trivial_tokens(&mut self) {
        let mode = self.newline_mode.clone();
        while is_trivial(self.tokens.peek().unwrap_or(&self.eof_token), &mode) {
            self.tokens.next();
        }
    }
}

impl From<Vec<Token>> for TokenCursor {
    /// Constructs a new cursor for a predefined set of tokens.
    fn from(tokens: Vec<Token>) -> Self {
        let start = tokens.first().map_or(0, |token| token.span.start);
        let end = tokens.last().map_or(start, |token| token.span.end);

        Self {
            eof_token: Token::new(TokenContents::Eof, Span::new(start, end)),
            tokens: tokens.into_iter().peekable(),
            newline_mode: NewlineMode::Newline,
        }
    }
}

/// Returns `true` if a [`Token`] is considered trivial in a [`NewlineMode`].
///
/// Trivial tokens are typically discarded.
fn is_trivial(token: &Token, newline_mode: &NewlineMode) -> bool {
    match token.contents {
        TokenContents::Comment | TokenContents::Whitespace => true,

        // Eol is trivialized when treating newline as whitespace.
        TokenContents::Eol if newline_mode == &NewlineMode::Whitespace => true,

        _ => false,
    }
}
