use pjsh_ast::Filter;

use super::{cursor::TokenCursor, word::parse_word, ParseResult};

/// Parses a condition.
pub(crate) fn parse_filter(tokens: &mut TokenCursor) -> ParseResult<Filter> {
    let name = parse_word(tokens)?;
    let mut args = Vec::new();

    while let Ok(arg) = parse_word(tokens) {
        args.push(arg);
    }

    Ok(Filter { name, args })
}

#[cfg(test)]
mod tests {
    use pjsh_ast::Word;

    use crate::{
        token::{Token, TokenContents},
        Span,
    };

    use super::*;

    /// Parses tokens into a filter.
    fn parse(contents: Vec<TokenContents>) -> ParseResult<Filter> {
        let span = Span::new(0, 0); // Insignificant during these tests.
        let tokens: Vec<Token> = contents.into_iter().map(|c| Token::new(c, span)).collect();
        let mut token_cursor = TokenCursor::from(tokens);
        parse_filter(&mut token_cursor)
    }

    #[test]
    fn it_parses_argumentless_filter() {
        assert_eq!(
            parse(vec![TokenContents::Literal("sort".into())]),
            Ok(Filter {
                name: Word::Literal("sort".into()),
                args: vec![]
            })
        );
    }

    #[test]
    fn it_parses_single_argument_filters() {
        assert_eq!(
            parse(vec![
                TokenContents::Literal("nth".into()),
                TokenContents::Literal("0".into()),
            ]),
            Ok(Filter {
                name: Word::Literal("nth".into()),
                args: vec![Word::Literal("0".into())]
            })
        );
    }

    #[test]
    fn it_parses_two_argument_filters() {
        assert_eq!(
            parse(vec![
                TokenContents::Literal("replace".into()),
                TokenContents::Literal("a".into()),
                TokenContents::Literal("b".into()),
            ]),
            Ok(Filter {
                name: Word::Literal("replace".into()),
                args: vec![Word::Literal("a".into()), Word::Literal("b".into())]
            })
        );
    }
}
