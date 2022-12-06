use pjsh_ast::{Command, FileDescriptor, Redirect, RedirectMode};

use crate::token::TokenContents;

use super::{cursor::TokenCursor, utils::unexpected_token, word::parse_word, ParseResult};

/// Tries to parse a [`Command`] from the next tokens of input.
pub fn parse_command(tokens: &mut TokenCursor) -> ParseResult<Command> {
    let prefix_redirects = parse_redirects(tokens);
    let mut command = Command::default();
    command.arg(parse_word(tokens)?);

    while let Ok(argument) = parse_word(tokens) {
        command.arg(argument);
    }

    for redirect in prefix_redirects {
        command.redirect(redirect);
    }
    for redirect in parse_redirects(tokens) {
        command.redirect(redirect);
    }

    Ok(command)
}

/// Parses a sequence of [`Redirect`] definitions.
/// Returns [`Vec::new()`] if the next non-trivial tokens are not valid redirects.
fn parse_redirects(tokens: &mut TokenCursor) -> Vec<Redirect> {
    let mut redirects = Vec::new();
    while let Ok(redirect) = parse_redirect(tokens) {
        redirects.push(redirect);
    }
    redirects
}

/// Parses a single redirect.
pub(crate) fn parse_redirect(tokens: &mut TokenCursor) -> ParseResult<Redirect> {
    match tokens.peek().contents {
        TokenContents::FdReadTo(fd) => {
            tokens.next();
            Ok(Redirect::new(
                FileDescriptor::File(parse_word(tokens)?),
                FileDescriptor::Number(fd),
                RedirectMode::Write,
            ))
        }
        TokenContents::FdWriteFrom(fd) => {
            tokens.next();
            Ok(Redirect::new(
                FileDescriptor::Number(fd),
                FileDescriptor::File(parse_word(tokens)?),
                RedirectMode::Write,
            ))
        }
        TokenContents::FdAppendFrom(fd) => {
            tokens.next();
            Ok(Redirect::new(
                FileDescriptor::Number(fd),
                FileDescriptor::File(parse_word(tokens)?),
                RedirectMode::Append,
            ))
        }
        _ => Err(unexpected_token(tokens)),
    }
}

#[cfg(test)]
mod tests {
    use pjsh_ast::Word;

    use crate::{token::Token, Span};

    use super::*;

    #[test]
    fn parse_redirect_read() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_redirect(&mut TokenCursor::from(vec![
                Token::new(TokenContents::FdReadTo(0), span),
                Token::new(TokenContents::Literal("file".into()), span),
            ])),
            Ok(Redirect {
                source: FileDescriptor::File(Word::Literal("file".into())),
                target: FileDescriptor::Number(0),
                mode: RedirectMode::Write
            })
        )
    }

    #[test]
    fn parse_redirect_write() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_redirect(&mut TokenCursor::from(vec![
                Token::new(TokenContents::FdWriteFrom(1), span),
                Token::new(TokenContents::Literal("file".into()), span),
            ])),
            Ok(Redirect {
                source: FileDescriptor::Number(1),
                target: FileDescriptor::File(Word::Literal("file".into())),
                mode: RedirectMode::Write
            })
        )
    }

    #[test]
    fn parse_redirect_append() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_redirect(&mut TokenCursor::from(vec![
                Token::new(TokenContents::FdAppendFrom(1), span),
                Token::new(TokenContents::Literal("file".into()), span),
            ])),
            Ok(Redirect {
                source: FileDescriptor::Number(1),
                target: FileDescriptor::File(Word::Literal("file".into())),
                mode: RedirectMode::Append
            })
        )
    }
}
