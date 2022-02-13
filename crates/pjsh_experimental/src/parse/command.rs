use crate::{
    ast::{Command, Word},
    input::Tokens,
    traits::{Parse, ParseResult},
};

struct CommandParser<'a> {
    word_parser: Box<dyn Parse<'a, Word<'a>>>,
}
impl<'a> Parse<'a, Command<'a>> for CommandParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Command<'a>> {
        let mut words = Vec::with_capacity(tokens.len());

        while let Ok(word) = self.word_parser.parse(tokens) {
            words.push(word);
        }

        Ok(Command(words))
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::{
        error::ParseError,
        token::{Span, Token, TokenContents},
    };

    use super::*;

    mock! {
        CommandParser {}
        impl Parse<'static, Word<'static>> for CommandParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Word<'static>>;
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
    fn it_parses_commands() {
        let mut word_parser = MockCommandParser::new();
        word_parser
            .expect_parse()
            .times(2)
            .returning(|_| Ok(Word::Literal("word")));
        word_parser
            .expect_parse()
            .returning(|_| Err(ParseError::UnexpectedToken));

        let parser = CommandParser {
            word_parser: Box::new(word_parser),
        };

        assert_eq!(
            Ok(Command(vec![Word::Literal("word"), Word::Literal("word")])),
            parse_command(
                Box::new(parser),
                vec![
                    TokenContents::Literal("word"),
                    TokenContents::Literal("word"),
                ]
            )
        );
    }
}
