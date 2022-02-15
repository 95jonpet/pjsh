use crate::{
    ast::{Expression, Pipeline},
    error::ParseError,
    input::Tokens,
    token::TokenContents,
    traits::{Parse, ParseResult},
};

struct ExpressionParser<'a> {
    pipeline_parser: Box<dyn Parse<'a, Pipeline<'a>>>,
}

impl<'a> ExpressionParser<'a> {
    fn parse_and_if(
        &self,
        tokens: &mut Tokens<'a>,
        root_node: Expression<'a>,
    ) -> ParseResult<Expression<'a>> {
        if tokens.next_if_eq(TokenContents::AndIf).is_none() {
            return Err(ParseError::UnexpectedToken);
        }

        let right_node = Expression::Pipeline(self.pipeline_parser.parse(tokens)?);
        Ok(Expression::And(Box::new(root_node), Box::new(right_node)))
    }

    fn parse_or_if(
        &self,
        tokens: &mut Tokens<'a>,
        root_node: Expression<'a>,
    ) -> ParseResult<Expression<'a>> {
        if tokens.next_if_eq(TokenContents::OrIf).is_none() {
            return Err(ParseError::UnexpectedToken);
        }

        let right_node = Expression::Pipeline(self.pipeline_parser.parse(tokens)?);
        Ok(Expression::Or(Box::new(root_node), Box::new(right_node)))
    }
}

impl<'a> Parse<'a, Expression<'a>> for ExpressionParser<'a> {
    fn parse(&self, tokens: &mut Tokens<'a>) -> ParseResult<Expression<'a>> {
        let mut root = Expression::Pipeline(self.pipeline_parser.parse(tokens)?);

        loop {
            match tokens.peek().it {
                TokenContents::AndIf => root = self.parse_and_if(tokens, root)?,
                TokenContents::OrIf => root = self.parse_or_if(tokens, root)?,
                TokenContents::Eol | TokenContents::Eof => break,
                _ => return Err(ParseError::UnexpectedToken),
            }
        }

        Ok(root)
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::{
        ast::{Command, Pipeline, PipelineSegment, Word},
        token::{Span, Token, TokenContents},
    };

    use super::*;

    mock! {
        ExpressionParser {}
        impl Parse<'static, Expression<'static>> for ExpressionParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Expression<'static>>;
        }
    }

    mock! {
        PipelineParser {}
        impl Parse<'static, Pipeline<'static>> for PipelineParser {
            fn parse(&self, tokens: &mut Tokens<'static>) -> ParseResult<Pipeline<'static>>;
        }
    }

    fn pipeline(name: &str) -> Pipeline {
        Pipeline {
            is_async: false,
            segments: vec![PipelineSegment::Command(Command(vec![Word::Literal(name)]))],
        }
    }

    fn parse_expression(
        parser: Box<dyn Parse<'static, Expression<'static>>>,
        tokens: Vec<TokenContents<'static>>,
    ) -> ParseResult<Expression<'static>> {
        let tokens: Vec<Token> = tokens
            .into_iter()
            .map(|contents| Token::new(Span::new(0, 0), contents))
            .collect();

        parser.parse(&mut Tokens::from(tokens))
    }

    #[test]
    fn it_parses_and_expressions() {
        let mut pipeline_parser = MockPipelineParser::new();

        pipeline_parser
            .expect_parse()
            .once()
            .returning(move |_| Ok(pipeline("A")));
        pipeline_parser
            .expect_parse()
            .once()
            .returning(move |_| Ok(pipeline("B")));

        let parser = ExpressionParser {
            pipeline_parser: Box::new(pipeline_parser),
        };

        assert_eq!(
            Ok(Expression::And(
                Box::new(Expression::Pipeline(pipeline("A"))),
                Box::new(Expression::Pipeline(pipeline("B")))
            )),
            parse_expression(Box::new(parser), vec![TokenContents::AndIf])
        );
    }

    #[test]
    fn it_parses_or_expressions() {
        let mut pipeline_parser = MockPipelineParser::new();

        pipeline_parser
            .expect_parse()
            .once()
            .returning(move |_| Ok(pipeline("A")));
        pipeline_parser
            .expect_parse()
            .once()
            .returning(move |_| Ok(pipeline("B")));

        let parser = ExpressionParser {
            pipeline_parser: Box::new(pipeline_parser),
        };

        assert_eq!(
            Ok(Expression::Or(
                Box::new(Expression::Pipeline(pipeline("A"))),
                Box::new(Expression::Pipeline(pipeline("B")))
            )),
            parse_expression(Box::new(parser), vec![TokenContents::OrIf])
        );
    }

    #[test]
    fn it_parses_complex_expressions() {
        let mut pipeline_parser = MockPipelineParser::new();

        pipeline_parser
            .expect_parse()
            .once()
            .returning(move |_| Ok(pipeline("A")));
        pipeline_parser
            .expect_parse()
            .once()
            .returning(move |_| Ok(pipeline("B")));
        pipeline_parser
            .expect_parse()
            .once()
            .returning(move |_| Ok(pipeline("C")));

        let parser = ExpressionParser {
            pipeline_parser: Box::new(pipeline_parser),
        };

        // Parse `A || B && C` into `And(Or(A, B), C)`.
        assert_eq!(
            Ok(Expression::And(
                Box::new(Expression::Or(
                    Box::new(Expression::Pipeline(pipeline("A"))),
                    Box::new(Expression::Pipeline(pipeline("B")))
                )),
                Box::new(Expression::Pipeline(pipeline("C"))),
            )),
            parse_expression(
                Box::new(parser),
                vec![TokenContents::OrIf, TokenContents::AndIf,]
            )
        );
    }
}
