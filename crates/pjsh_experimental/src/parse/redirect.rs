use crate::{
    ast::{FileDescriptor, Redirect, RedirectMethod, Word},
    error::ParseError,
    input::Tokens,
    token::TokenContents,
    traits::{Parse, ParseResult},
};

struct RedirectParser<'a> {
    word_parser: Box<dyn Parse<'a, Word<'a>>>,
}

impl<'a> RedirectParser<'a> {
    pub fn new(word_parser: Box<dyn Parse<'a, Word<'a>>>) -> Self {
        Self { word_parser }
    }
}

impl<'a> Parse<'a, Redirect<'a>> for RedirectParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Redirect<'a>> {
        match tokens.peek().it {
            TokenContents::FdReadTo(fd) => {
                tokens.next();
                Ok(Redirect {
                    source: FileDescriptor::Named(self.word_parser.parse(tokens)?),
                    method: RedirectMethod::Write,
                    target: FileDescriptor::Numbered(fd),
                })
            }
            TokenContents::FdWriteFrom(fd) => {
                tokens.next();
                Ok(Redirect {
                    source: FileDescriptor::Numbered(fd),
                    method: RedirectMethod::Write,
                    target: FileDescriptor::Named(self.word_parser.parse(tokens)?),
                })
            }
            TokenContents::FdAppendFrom(fd) => {
                tokens.next();
                Ok(Redirect {
                    source: FileDescriptor::Numbered(fd),
                    method: RedirectMethod::Append,
                    target: FileDescriptor::Named(self.word_parser.parse(tokens)?),
                })
            }
            _ => Err(ParseError::UnexpectedToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::token::{Span, Token, TokenContents};

    use super::*;

    mock! {
        WordParser {}
        impl Parse<'static, Word<'static>> for WordParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Word<'static>>;
        }
    }

    fn parse_redirect(
        parser: Box<dyn Parse<'static, Redirect<'static>>>,
        tokens: Vec<TokenContents<'static>>,
    ) -> ParseResult<Redirect<'static>> {
        let tokens: Vec<Token> = tokens
            .into_iter()
            .map(|contents| Token::new(Span::new(0, 0), contents))
            .collect();

        parser.parse(&mut Tokens::from(tokens))
    }

    #[test]
    fn it_parses_redirect_write_from_numbered_to_named_fd() {
        let mut word_parser = MockWordParser::new();
        word_parser
            .expect_parse()
            .return_once(|_| Ok(Word::Literal("target")));

        let parser = RedirectParser {
            word_parser: Box::new(word_parser),
        };

        assert_eq!(
            Ok(Redirect {
                source: FileDescriptor::Numbered(1),
                method: RedirectMethod::Write,
                target: FileDescriptor::Named(Word::Literal("target")),
            }),
            parse_redirect(
                Box::new(parser),
                vec![
                    TokenContents::FdWriteFrom(1),
                    TokenContents::Literal("target"),
                ]
            )
        );
    }

    #[test]
    fn it_parses_redirect_append_from_numbered_to_named_fd() {
        let mut word_parser = MockWordParser::new();
        word_parser
            .expect_parse()
            .return_once(|_| Ok(Word::Literal("target")));

        let parser = RedirectParser {
            word_parser: Box::new(word_parser),
        };

        assert_eq!(
            Ok(Redirect {
                source: FileDescriptor::Numbered(2),
                method: RedirectMethod::Append,
                target: FileDescriptor::Named(Word::Literal("target")),
            }),
            parse_redirect(
                Box::new(parser),
                vec![
                    TokenContents::FdAppendFrom(2),
                    TokenContents::Literal("target"),
                ]
            )
        );
    }

    #[test]
    fn it_parses_redirect_read_from_named_to_numbered_fd() {
        let mut word_parser = MockWordParser::new();
        word_parser
            .expect_parse()
            .return_once(|_| Ok(Word::Literal("source")));

        let parser = RedirectParser {
            word_parser: Box::new(word_parser),
        };

        assert_eq!(
            Ok(Redirect {
                source: FileDescriptor::Named(Word::Literal("source")),
                method: RedirectMethod::Write,
                target: FileDescriptor::Numbered(0),
            }),
            parse_redirect(
                Box::new(parser),
                vec![TokenContents::FdReadTo(0), TokenContents::Literal("source")]
            )
        );
    }
}
