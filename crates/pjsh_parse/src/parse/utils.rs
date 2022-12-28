use crate::{
    token::{Token, TokenContents},
    ParseError,
};

use super::{cursor::TokenCursor, ParseResult};

/// Does everything inside a closure and advance the cursor.
///
/// This function assumes that everything inside the closure is related and that
/// its contents should be evaluated sequentially.
pub fn sequence<F, T>(tokens: &mut TokenCursor, func: F) -> ParseResult<T>
where
    F: Fn(&mut TokenCursor) -> ParseResult<T>,
{
    let mut peek = tokens.clone();
    match func(&mut peek) {
        Ok(value) => {
            *tokens = peek;
            Ok(value)
        }
        Err(ParseError::UnexpectedEof) => Err(ParseError::IncompleteSequence),
        error => error,
    }
}

/// Advances the token cursor until the next token is not an end-of-line token.
pub fn skip_newlines(tokens: &mut TokenCursor) {
    while matches!(
        tokens.peek().contents,
        TokenContents::Eol | TokenContents::Semi
    ) {
        tokens.next();
    }
}

/// Advances past a literal if the next matches the given literal.
/// Returns an error if the next token is unexpected.
pub fn take_literal(tokens: &mut TokenCursor, literal: &str) -> ParseResult<Token> {
    let token = tokens
        .next_if(|token| matches!(&token.contents, TokenContents::Literal(it) if it == literal));

    token.ok_or_else(|| unexpected_token(tokens))
}

/// Advances past a token if the next matches the given contents.
/// Returns an error if the next token is unexpected.
pub fn take_token(tokens: &mut TokenCursor, contents: &TokenContents) -> ParseResult<Token> {
    tokens
        .next_if(|token| &token.contents == contents)
        .ok_or_else(|| expected_token(tokens, contents.clone()))
}

/// Returns a [`ParseError::UnexpectedToken`] around a copy of the next token.
pub fn expected_token(tokens: &mut TokenCursor, expected: TokenContents) -> ParseError {
    ParseError::ExpectedToken(expected, tokens.peek().clone())
}

/// Returns a [`ParseError::UnexpectedToken`] around a copy of the next token.
pub fn unexpected_token(tokens: &mut TokenCursor) -> ParseError {
    ParseError::UnexpectedToken(tokens.peek().clone())
}
