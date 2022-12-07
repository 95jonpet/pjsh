use pjsh_ast::{Command, FileDescriptor, Redirect, RedirectMode};

use crate::token::TokenContents;

use super::{cursor::TokenCursor, utils::unexpected_token, word::parse_word, ParseResult};

/// Tries to parse a [`Command`] from the next tokens of input.
pub fn parse_command(tokens: &mut TokenCursor) -> ParseResult<Command> {
    let mut command = Command::default();
    command.redirects.extend(parse_redirects(tokens)); // Prefix redirects.

    // A command must include at least one argument denoting the program name.
    command.arg(parse_word(tokens)?);

    // Additional arguments are optional.
    while let Ok(argument) = parse_word(tokens) {
        command.arg(argument);
    }

    command.redirects.extend(parse_redirects(tokens)); // Suffix redirects.

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
    fn parse_single_argument_command() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_command(&mut TokenCursor::from(vec![Token::new(
                TokenContents::Literal("program".into()),
                span
            )])),
            Ok(Command {
                arguments: vec![Word::Literal("program".into())],
                redirects: Vec::new(),
            })
        )
    }

    #[test]
    fn parse_muli_argument_command() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_command(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("program".into()), span),
                Token::new(TokenContents::Literal("arg".into()), span),
            ])),
            Ok(Command {
                arguments: vec![Word::Literal("program".into()), Word::Literal("arg".into()),],
                redirects: Vec::new(),
            })
        )
    }

    #[test]
    fn parse_command_with_prefix_redirects() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_command(&mut TokenCursor::from(vec![
                Token::new(TokenContents::FdReadTo(0), span),
                Token::new(TokenContents::Literal("prefix1".into()), span),
                Token::new(TokenContents::FdWriteFrom(1), span),
                Token::new(TokenContents::Literal("prefix2".into()), span),
                Token::new(TokenContents::Literal("program".into()), span),
            ])),
            Ok(Command {
                arguments: vec![Word::Literal("program".into())],
                redirects: vec![
                    Redirect {
                        source: FileDescriptor::File(Word::Literal("prefix1".into())),
                        target: FileDescriptor::Number(0),
                        mode: RedirectMode::Write
                    },
                    Redirect {
                        source: FileDescriptor::Number(1),
                        target: FileDescriptor::File(Word::Literal("prefix2".into())),
                        mode: RedirectMode::Write
                    },
                ],
            })
        )
    }

    #[test]
    fn parse_command_with_suffix_redirects() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_command(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("program".into()), span),
                Token::new(TokenContents::FdReadTo(0), span),
                Token::new(TokenContents::Literal("suffix1".into()), span),
                Token::new(TokenContents::FdWriteFrom(1), span),
                Token::new(TokenContents::Literal("suffix2".into()), span),
            ])),
            Ok(Command {
                arguments: vec![Word::Literal("program".into())],
                redirects: vec![
                    Redirect {
                        source: FileDescriptor::File(Word::Literal("suffix1".into())),
                        target: FileDescriptor::Number(0),
                        mode: RedirectMode::Write
                    },
                    Redirect {
                        source: FileDescriptor::Number(1),
                        target: FileDescriptor::File(Word::Literal("suffix2".into())),
                        mode: RedirectMode::Write
                    },
                ],
            })
        )
    }

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
