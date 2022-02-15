use crate::{
    ast::{Command, Pipeline},
    error::ParseError,
    input::Tokens,
    token::TokenContents,
    traits::{Parse, ParseResult},
};

struct LegacyPipelineParser<'a> {
    command_parser: Box<dyn Parse<'a, Command<'a>>>,
}

impl<'a> LegacyPipelineParser<'a> {
    pub fn new(command_parser: Box<dyn Parse<'a, Command<'a>>>) -> Self {
        Self { command_parser }
    }
}

impl<'a> Parse<'a, Pipeline<'a>> for LegacyPipelineParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Pipeline<'a>> {
        let mut commands = Vec::with_capacity(tokens.len());

        // No input to parse - a valid legacy pipeline cannot be constructed.
        if tokens.peek().it == TokenContents::Eof {
            return Err(ParseError::UnexpectedToken);
        }

        commands.push(self.command_parser.parse(tokens)?);

        while tokens.next_if_eq(TokenContents::Pipe).is_some() {
            commands.push(self.command_parser.parse(tokens)?);
        }

        let mut is_async = false;
        if tokens.next_if_eq(TokenContents::Amp).is_some() {
            is_async = true;
        }

        match tokens.peek().it {
            TokenContents::Eol | TokenContents::Eof => Ok(Pipeline { is_async, commands }),
            _ => Err(ParseError::UnexpectedToken),
        }
    }
}

struct SmartPipelineParser<'a> {
    command_parser: Box<dyn Parse<'a, Command<'a>>>,
}

impl<'a> SmartPipelineParser<'a> {
    pub fn new(command_parser: Box<dyn Parse<'a, Command<'a>>>) -> Self {
        Self { command_parser }
    }
}

impl<'a> Parse<'a, Pipeline<'a>> for SmartPipelineParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Pipeline<'a>> {
        if tokens.next_if_eq(TokenContents::PipeStart).is_none() {
            return Err(ParseError::UnexpectedToken);
        }

        let mut commands = Vec::with_capacity(tokens.len());
        let mut is_async = false;

        if tokens.peek().it == TokenContents::Eof {
            return Err(ParseError::IncompleteSequence);
        }

        commands.push(self.command_parser.parse(tokens)?);

        while tokens.next_if_eq(TokenContents::Pipe).is_some() {
            commands.push(self.command_parser.parse(tokens)?);
        }

        match tokens.peek().it {
            TokenContents::Eof => return Err(ParseError::IncompleteSequence),
            TokenContents::Amp => {
                tokens.next();
                is_async = true;
            }
            TokenContents::Semi => {
                tokens.next();
            }
            _ => return Err(ParseError::UnexpectedToken),
        }

        Ok(Pipeline { is_async, commands })
    }
}

struct PipelineParser<'a> {
    legacy_pipeline_parser: Box<dyn Parse<'a, Pipeline<'a>>>,
    smart_pipeline_parser: Box<dyn Parse<'a, Pipeline<'a>>>,
}

impl<'a> PipelineParser<'a> {
    pub fn new(
        legacy_pipeline_parser: Box<dyn Parse<'a, Pipeline<'a>>>,
        smart_pipeline_parser: Box<dyn Parse<'a, Pipeline<'a>>>,
    ) -> Self {
        Self {
            legacy_pipeline_parser,
            smart_pipeline_parser,
        }
    }
}

impl<'a> Parse<'a, Pipeline<'a>> for PipelineParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Pipeline<'a>> {
        if tokens.peek().it == TokenContents::PipeStart {
            return self.smart_pipeline_parser.parse(tokens);
        }

        self.legacy_pipeline_parser.parse(tokens)
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::{
        ast::Word,
        token::{Span, Token, TokenContents},
    };

    use super::*;

    mock! {
        CommandParser {}
        impl Parse<'static, Command<'static>> for CommandParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Command<'static>>;
        }
    }

    mock! {
        PipelineParser {}
        impl Parse<'static, Pipeline<'static>> for PipelineParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Pipeline<'static>>;
        }
    }

    fn parse_pipeline(
        parser: Box<dyn Parse<'static, Pipeline<'static>>>,
        tokens: Vec<TokenContents<'static>>,
    ) -> ParseResult<Pipeline<'static>> {
        let tokens: Vec<Token> = tokens
            .into_iter()
            .map(|contents| Token::new(Span::new(0, 0), contents))
            .collect();

        parser.parse(&mut Tokens::from(tokens))
    }

    #[test]
    fn it_parses_legacy_pipelines() {
        let mut command_parser = MockCommandParser::new();
        command_parser
            .expect_parse()
            .times(2)
            .returning(|_| Ok(Command(vec![Word::Literal("command")])));
        command_parser
            .expect_parse()
            .return_once(|_| Err(ParseError::UnexpectedToken));

        let parser = LegacyPipelineParser::new(Box::new(command_parser));

        assert_eq!(
            Ok(Pipeline {
                is_async: false,
                commands: vec![
                    Command(vec![Word::Literal("command")]),
                    Command(vec![Word::Literal("command")])
                ]
            }),
            parse_pipeline(
                Box::new(parser),
                vec![TokenContents::Pipe, TokenContents::Eol]
            )
        );
    }

    #[test]
    fn it_parses_async_legacy_pipelines() {
        let mut command_parser = MockCommandParser::new();
        command_parser
            .expect_parse()
            .times(2)
            .returning(|_| Ok(Command(vec![Word::Literal("command")])));
        command_parser
            .expect_parse()
            .return_once(|_| Err(ParseError::UnexpectedToken));

        let parser = LegacyPipelineParser::new(Box::new(command_parser));

        assert_eq!(
            Ok(Pipeline {
                is_async: true,
                commands: vec![
                    Command(vec![Word::Literal("command")]),
                    Command(vec![Word::Literal("command")])
                ]
            }),
            parse_pipeline(
                Box::new(parser),
                vec![TokenContents::Pipe, TokenContents::Amp, TokenContents::Eol]
            )
        );
    }

    #[test]
    fn it_parses_smart_pipelines() {
        let mut command_parser = MockCommandParser::new();
        command_parser
            .expect_parse()
            .times(2)
            .returning(|_| Ok(Command(vec![Word::Literal("command")])));
        command_parser
            .expect_parse()
            .return_once(|_| Err(ParseError::UnexpectedToken));

        let parser = SmartPipelineParser::new(Box::new(command_parser));

        assert_eq!(
            Ok(Pipeline {
                is_async: false,
                commands: vec![
                    Command(vec![Word::Literal("command")]),
                    Command(vec![Word::Literal("command")])
                ]
            }),
            parse_pipeline(
                Box::new(parser),
                vec![
                    TokenContents::PipeStart,
                    TokenContents::Pipe,
                    TokenContents::Semi
                ]
            )
        );
    }

    #[test]
    fn it_parses_async_smart_pipelines() {
        let mut command_parser = MockCommandParser::new();
        command_parser
            .expect_parse()
            .times(2)
            .returning(|_| Ok(Command(vec![Word::Literal("command")])));
        command_parser
            .expect_parse()
            .return_once(|_| Err(ParseError::UnexpectedToken));

        let parser = SmartPipelineParser::new(Box::new(command_parser));

        assert_eq!(
            Ok(Pipeline {
                is_async: true,
                commands: vec![
                    Command(vec![Word::Literal("command")]),
                    Command(vec![Word::Literal("command")])
                ]
            }),
            parse_pipeline(
                Box::new(parser),
                vec![
                    TokenContents::PipeStart,
                    TokenContents::Pipe,
                    TokenContents::Amp,
                ]
            )
        );
    }

    #[test]
    fn it_parses_legacy_pipelines_using_a_subparser() {
        let mut legacy_pipeline_parser = MockPipelineParser::new();
        legacy_pipeline_parser.expect_parse().return_once(|_| {
            Ok(Pipeline {
                is_async: false,
                commands: vec![
                    Command(vec![Word::Literal("command")]),
                    Command(vec![Word::Literal("command")]),
                ],
            })
        });

        let parser = PipelineParser::new(
            Box::new(legacy_pipeline_parser),
            Box::new(MockPipelineParser::new()),
        );

        assert_eq!(
            Ok(Pipeline {
                is_async: false,
                commands: vec![
                    Command(vec![Word::Literal("command")]),
                    Command(vec![Word::Literal("command")]),
                ],
            }),
            parse_pipeline(Box::new(parser), vec![]) // No smart pipeline start, parse legacy pipeline.
        );
    }

    #[test]
    fn it_parses_smart_pipelines_using_a_subparser() {
        let mut smart_pipeline_parser = MockPipelineParser::new();
        smart_pipeline_parser.expect_parse().return_once(|_| {
            Ok(Pipeline {
                is_async: false,
                commands: vec![
                    Command(vec![Word::Literal("command")]),
                    Command(vec![Word::Literal("command")]),
                ],
            })
        });

        let parser = PipelineParser::new(
            Box::new(MockPipelineParser::new()),
            Box::new(smart_pipeline_parser),
        );

        assert_eq!(
            Ok(Pipeline {
                is_async: false,
                commands: vec![
                    Command(vec![Word::Literal("command")]),
                    Command(vec![Word::Literal("command")]),
                ],
            }),
            parse_pipeline(Box::new(parser), vec![TokenContents::PipeStart])
        );
    }
}
