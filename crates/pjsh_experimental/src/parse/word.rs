use crate::{
    ast::Word,
    error::ParseError,
    input::Tokens,
    token::TokenContents,
    traits::{Parse, ParseResult},
};

struct WordParser<'a> {
    quoted_word_parser: Box<dyn Parse<'a, Word<'a>>>,
    triple_quoted_word_parser: Box<dyn Parse<'a, Word<'a>>>,
}

impl<'a> WordParser<'a> {
    pub fn new(
        quoted_word_parser: Box<dyn Parse<'a, Word<'a>>>,
        triple_quoted_word_parser: Box<dyn Parse<'a, Word<'a>>>,
    ) -> Self {
        Self {
            quoted_word_parser,
            triple_quoted_word_parser,
        }
    }
}

impl<'a> Parse<'a, Word<'a>> for WordParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Word<'a>> {
        let token = tokens.peek();
        match token.it {
            TokenContents::Literal(word) => {
                tokens.next();
                Ok(Word::Literal(word))
            }
            TokenContents::Variable(word) => {
                tokens.next();
                Ok(Word::Variable(word))
            }
            TokenContents::Quote => self.quoted_word_parser.parse(tokens),
            TokenContents::TripleQuote => self.triple_quoted_word_parser.parse(tokens),
            _ => Err(ParseError::UnexpectedToken),
        }
    }
}

struct TripleQuotedWordParser;
impl<'a> Parse<'a, Word<'a>> for TripleQuotedWordParser {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Word<'a>> {
        if tokens.next_if_eq(TokenContents::TripleQuote).is_none() {
            return Err(ParseError::UnexpectedToken);
        }

        if let TokenContents::TripleQuoted(word) = tokens.next().it {
            return match tokens.peek().it {
                TokenContents::TripleQuote => {
                    tokens.next();
                    Ok(Word::TripleQuoted(word))
                }
                TokenContents::Eof => Err(ParseError::IncompleteSequence),
                _ => Err(ParseError::UnexpectedToken),
            };
        }

        Err(ParseError::IncompleteSequence)
    }
}

struct QuotedWordParser;
impl<'a> Parse<'a, Word<'a>> for QuotedWordParser {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Word<'a>> {
        if tokens.next_if_eq(TokenContents::Quote).is_none() {
            return Err(ParseError::UnexpectedToken);
        }

        if let TokenContents::Quoted(word) = tokens.next().it {
            return match tokens.peek().it {
                TokenContents::Quote => {
                    tokens.next();
                    Ok(Word::Quoted(word))
                }
                TokenContents::Eof => Err(ParseError::IncompleteSequence),
                _ => Err(ParseError::UnexpectedToken),
            };
        }

        Err(ParseError::IncompleteSequence)
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::token::{Span, Token};

    use super::*;

    mock! {
        QuotedWordParser {}
        impl Parse<'static, Word<'static>> for QuotedWordParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Word<'static>>;
        }
    }

    mock! {
        TripleQuotedWordParser {}
        impl Parse<'static, Word<'static>> for TripleQuotedWordParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Word<'static>>;
        }
    }

    fn parse_word(
        parser: Box<dyn Parse<'static, Word<'static>>>,
        tokens: Vec<TokenContents<'static>>,
    ) -> ParseResult<Word<'static>> {
        let tokens: Vec<Token> = tokens
            .into_iter()
            .map(|contents| Token::new(Span::new(0, 0), contents))
            .collect();

        parser.parse(&mut Tokens::from(tokens))
    }

    #[test]
    fn it_parses_literals() {
        let parser = WordParser::new(
            Box::new(MockQuotedWordParser::new()),
            Box::new(MockTripleQuotedWordParser::new()),
        );

        assert_eq!(
            Ok(Word::Literal("word")),
            parse_word(Box::new(parser), vec![TokenContents::Literal("word")])
        );
    }

    #[test]
    fn it_parses_quoted_words() {
        let parser = QuotedWordParser;

        assert_eq!(
            Ok(Word::Quoted("quoted")),
            parse_word(
                Box::new(parser),
                vec![
                    TokenContents::Quote,
                    TokenContents::Quoted("quoted"),
                    TokenContents::Quote,
                ]
            )
        );
    }

    #[test]
    fn it_parses_triple_quoted_words() {
        let parser = TripleQuotedWordParser;

        assert_eq!(
            Ok(Word::TripleQuoted("triple quoted")),
            parse_word(
                Box::new(parser),
                vec![
                    TokenContents::TripleQuote,
                    TokenContents::TripleQuoted("triple quoted"),
                    TokenContents::TripleQuote,
                ]
            )
        );
    }

    #[test]
    fn it_rejects_unmatching_quotes() {
        assert_eq!(
            Err(ParseError::IncompleteSequence),
            parse_word(Box::new(QuotedWordParser), vec![TokenContents::Quote])
        );

        assert_eq!(
            Err(ParseError::IncompleteSequence),
            parse_word(
                Box::new(QuotedWordParser),
                vec![TokenContents::Quote, TokenContents::Quoted("quoted")]
            )
        );
    }

    #[test]
    fn it_uses_wrapped_parser_for_quoted_words() {
        let mut quoted_word_parser = MockQuotedWordParser::new();
        quoted_word_parser
            .expect_parse()
            .return_once(|_| Ok(Word::Quoted("quoted")));

        let parser = WordParser::new(
            Box::new(quoted_word_parser),
            Box::new(MockTripleQuotedWordParser::new()),
        );

        assert_eq!(
            Ok(Word::Quoted("quoted")),
            parse_word(
                Box::new(parser),
                vec![
                    TokenContents::Quote,
                    TokenContents::Quoted("quoted"),
                    TokenContents::Quote,
                ]
            )
        );
    }

    #[test]
    fn it_uses_wrapped_parser_for_triple_quoted_words() {
        let mut triple_quoted_word_parser = MockTripleQuotedWordParser::new();
        triple_quoted_word_parser
            .expect_parse()
            .return_once(|_| Ok(Word::TripleQuoted("triple quoted")));

        let parser = WordParser::new(
            Box::new(MockQuotedWordParser::new()),
            Box::new(triple_quoted_word_parser),
        );

        assert_eq!(
            Ok(Word::TripleQuoted("triple quoted")),
            parse_word(
                Box::new(parser),
                vec![
                    TokenContents::TripleQuote,
                    TokenContents::TripleQuoted("triple quoted"),
                    TokenContents::TripleQuote,
                ]
            )
        );
    }
}
