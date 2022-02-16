use crate::{
    ast::{Command, Redirect, Word},
    input::Tokens,
    traits::{Parse, ParseResult},
};

struct CommandParser<'a> {
    argument_parser: Box<dyn Parse<'a, Word<'a>>>,
    redirect_parser: Box<dyn Parse<'a, Redirect<'a>>>,
}
impl<'a> Parse<'a, Command<'a>> for CommandParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Command<'a>> {
        let mut redirects = Vec::with_capacity(tokens.len());
        while let Ok(redirect) = self.redirect_parser.parse(tokens) {
            redirects.push(redirect);
        }

        let mut arguments = Vec::with_capacity(tokens.len());

        arguments.push(self.argument_parser.parse(tokens)?);
        while let Ok(argument) = self.argument_parser.parse(tokens) {
            arguments.push(argument);
        }

        while let Ok(redirect) = self.redirect_parser.parse(tokens) {
            redirects.push(redirect);
        }

        Ok(Command {
            arguments,
            redirects,
        })
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::{
        ast::{FileDescriptor, RedirectMethod},
        error::ParseError,
        token::{Span, Token, TokenContents},
    };

    use super::*;

    mock! {
        WordParser {}
        impl Parse<'static, Word<'static>> for WordParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Word<'static>>;
        }
    }

    mock! {
        RedirectParser {}
        impl Parse<'static, Redirect<'static>> for RedirectParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Redirect<'static>>;
        }
    }

    fn parse_command(
        parser: Box<dyn Parse<'static, Command<'static>>>,
        tokens: Vec<TokenContents<'static>>,
    ) -> ParseResult<Command<'static>> {
        let tokens: Vec<Token> = tokens
            .into_iter()
            .map(|contents| Token::new(Span::new(0, 0), contents))
            .collect();

        parser.parse(&mut Tokens::from(tokens))
    }

    #[test]
    fn it_parses_command_arguments() {
        let mut word_parser = MockWordParser::new();
        word_parser
            .expect_parse()
            .times(2)
            .returning(|_| Ok(Word::Literal("word")));
        word_parser
            .expect_parse()
            .returning(|_| Err(ParseError::UnexpectedToken));

        let mut redirect_parser = MockRedirectParser::new();
        redirect_parser
            .expect_parse()
            .returning(|_| Err(ParseError::UnexpectedToken));

        let parser = CommandParser {
            argument_parser: Box::new(word_parser),
            redirect_parser: Box::new(redirect_parser),
        };

        assert_eq!(
            Ok(Command {
                arguments: vec![Word::Literal("word"), Word::Literal("word")],
                redirects: vec![]
            }),
            parse_command(Box::new(parser), vec![])
        );
    }

    #[test]
    fn it_parses_prefixed_redirects() {
        let mut word_parser = MockWordParser::new();
        word_parser
            .expect_parse()
            .once()
            .returning(|_| Ok(Word::Literal("command")));
        word_parser
            .expect_parse()
            .returning(|_| Err(ParseError::UnexpectedToken));

        let mut redirect_parser = MockRedirectParser::new();
        redirect_parser.expect_parse().once().returning(|_| {
            Ok(Redirect {
                source: FileDescriptor::Numbered(1),
                target: FileDescriptor::Named(Word::Literal("/dev/nulll")),
                method: RedirectMethod::Write,
            })
        });
        redirect_parser
            .expect_parse()
            .returning(|_| Err(ParseError::UnexpectedToken));

        let parser = CommandParser {
            argument_parser: Box::new(word_parser),
            redirect_parser: Box::new(redirect_parser),
        };

        assert_eq!(
            Ok(Command {
                arguments: vec![Word::Literal("command")],
                redirects: vec![Redirect {
                    source: FileDescriptor::Numbered(1),
                    target: FileDescriptor::Named(Word::Literal("/dev/nulll")),
                    method: RedirectMethod::Write,
                }]
            }),
            parse_command(Box::new(parser), vec![])
        );
    }

    #[test]
    fn it_parses_postfixed_redirects() {
        let mut word_parser = MockWordParser::new();
        word_parser
            .expect_parse()
            .once()
            .returning(|_| Ok(Word::Literal("command")));
        word_parser
            .expect_parse()
            .returning(|_| Err(ParseError::UnexpectedToken));

        let mut redirect_parser = MockRedirectParser::new();
        redirect_parser
            .expect_parse()
            .once()
            .returning(|_| Err(ParseError::UnexpectedToken));
        redirect_parser.expect_parse().once().returning(|_| {
            Ok(Redirect {
                source: FileDescriptor::Numbered(1),
                target: FileDescriptor::Named(Word::Literal("/dev/nulll")),
                method: RedirectMethod::Write,
            })
        });
        redirect_parser
            .expect_parse()
            .once()
            .returning(|_| Err(ParseError::UnexpectedToken));

        let parser = CommandParser {
            argument_parser: Box::new(word_parser),
            redirect_parser: Box::new(redirect_parser),
        };

        assert_eq!(
            Ok(Command {
                arguments: vec![Word::Literal("command")],
                redirects: vec![Redirect {
                    source: FileDescriptor::Numbered(1),
                    target: FileDescriptor::Named(Word::Literal("/dev/nulll")),
                    method: RedirectMethod::Write,
                }]
            }),
            parse_command(Box::new(parser), vec![])
        );
    }
}
