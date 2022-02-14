use crate::{
    ast::{Condition, Word},
    error::ParseError,
    input::Tokens,
    token::TokenContents,
    traits::{Parse, ParseResult},
};

struct ConditionParser<'a> {
    word_parser: Box<dyn Parse<'a, Word<'a>>>,
}
impl<'a> Parse<'a, Condition<'a>> for ConditionParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Condition<'a>> {
        if tokens
            .next_if_eq(TokenContents::DoubleOpenBracket)
            .is_none()
        {
            return Err(ParseError::UnexpectedToken);
        }

        let mut words = Vec::with_capacity(tokens.len());

        while let Ok(word) = self.word_parser.parse(tokens) {
            words.push(word);
        }

        if tokens.peek().it == TokenContents::Eof {
            return Err(ParseError::IncompleteSequence);
        }

        if tokens
            .next_if_eq(TokenContents::DoubleCloseBracket)
            .is_none()
        {
            return Err(ParseError::UnexpectedToken);
        }

        Ok(Condition(words))
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
        WordParser {}
        impl Parse<'static, Word<'static>> for WordParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Word<'static>>;
        }
    }

    fn parse_condition(
        parser: Box<dyn Parse<'static, Condition<'static>>>,
        tokens: Vec<TokenContents<'static>>,
    ) -> ParseResult<Condition<'static>> {
        let tokens: Vec<Token> = tokens
            .into_iter()
            .map(|contents| Token::new(Span::new(0, 0), contents))
            .collect();

        parser.parse(&mut Tokens::from(tokens))
    }

    #[test]
    fn it_parses_conditions() {
        let mut word_parser = MockWordParser::new();
        word_parser
            .expect_parse()
            .times(2)
            .returning(|_| Ok(Word::Literal("word")));
        word_parser
            .expect_parse()
            .returning(|_| Err(ParseError::UnexpectedToken));

        let parser = ConditionParser {
            word_parser: Box::new(word_parser),
        };

        assert_eq!(
            Ok(Condition(vec![
                Word::Literal("word"),
                Word::Literal("word")
            ])),
            parse_condition(
                Box::new(parser),
                vec![
                    TokenContents::DoubleOpenBracket,
                    // Mock TokenContents::Literal("word"),
                    // Mock TokenContents::Literal("word"),
                    TokenContents::DoubleCloseBracket,
                ]
            )
        );
    }
}
