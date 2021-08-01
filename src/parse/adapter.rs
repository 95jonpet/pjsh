use std::{collections::VecDeque, mem::replace};

use crate::{
    lexer::{Lex, Mode},
    token::Token,
};

const DEFAULT_LEXER_MODE_STACK_CAPACITY: usize = 10;

pub struct LexerAdapter {
    lexer: Box<dyn Lex>,
    lexer_mode_stack: Vec<Mode>,
    cached_tokens: VecDeque<Token>,
}

impl LexerAdapter {
    pub fn new(lexer: Box<dyn Lex>) -> Self {
        let mut lexer_mode_stack = Vec::with_capacity(DEFAULT_LEXER_MODE_STACK_CAPACITY);
        lexer_mode_stack.push(Mode::Unquoted);

        Self {
            lexer,
            lexer_mode_stack,
            cached_tokens: VecDeque::new(),
        }
    }

    /// Returns all cached tokens and clears the cache.
    pub fn clear_cache(&mut self) -> VecDeque<Token> {
        replace(&mut self.cached_tokens, VecDeque::new())
    }

    /// Returns the current [`Mode`] that should be used when performing lexical analysis.
    pub fn lexer_mode(&self) -> Mode {
        *self
            .lexer_mode_stack
            .last()
            .expect("a lexer mode to be set")
    }

    /// Returns the next [`Token`] from the [`Lex`].
    /// The token is also cached locally.
    pub fn peek_token(&mut self) -> &Token {
        if self.cached_tokens.is_empty() {
            let next_token = self.lexer.next_token(self.lexer_mode());
            self.cached_tokens.push_back(next_token);
        }

        self.cached_tokens.front().unwrap_or(&Token::EOF)
    }

    /// Returns the next [`Token`] from the [`Lex`].
    /// Tokens may be locally cached if peeked .
    /// If the next token resides in the cache, it is also removed from the cache.
    pub fn next_token(&mut self) -> Token {
        self.cached_tokens
            .pop_front()
            .unwrap_or_else(|| self.lexer.next_token(self.lexer_mode()))
    }

    pub fn advance_line(&mut self) {
        if self.cached_tokens.iter().any(|token| token != &Token::EOF) {
            unreachable!();
        }

        self.cached_tokens.clear();
        self.lexer.advance_line()
    }

    // /// Set the current [`Lex`] mode.
    // pub fn push_lexer_mode(&mut self, lexer_mode: Mode) {
    //     if lexer_mode != self.lexer_mode() && !self.cached_tokens.is_empty() {
    //         unreachable!("The lexer mode should not be changed while peeked tokens are held!");
    //     }

    //     self.lexer_mode_stack.push(lexer_mode);
    // }

    // /// Restore the previous [`Lex`] mode.
    // pub fn pop_lexer_mode(&mut self) {
    //     self.lexer_mode_stack
    //         .pop()
    //         .expect("an empty lexer mode stack should not be popped");
    // }
}
