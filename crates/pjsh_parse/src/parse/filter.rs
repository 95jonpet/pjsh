use pjsh_ast::{Filter, Word};

use crate::ParseError;

use super::{cursor::TokenCursor, utils::take_literal, word::parse_word, ParseResult};

/// Parses a condition.
pub(crate) fn parse_filter(tokens: &mut TokenCursor) -> ParseResult<Filter> {
    Result::Err(ParseError::InvalidSyntax("Unknown filter".into()))
        .or_else(|_| argumentless_filter(tokens, "lower", Filter::Lower))
        .or_else(|_| argumentless_filter(tokens, "upper", Filter::Upper))
        .or_else(|_| argumentless_filter(tokens, "unique", Filter::Unique))
        .or_else(|_| argumentless_filter(tokens, "len", Filter::Len))
        .or_else(|_| argumentless_filter(tokens, "reverse", Filter::Reverse))
        .or_else(|_| argumentless_filter(tokens, "sort", Filter::Sort))
        .or_else(|_| one_argument_filter(tokens, "index", Filter::Index))
}

/// Returns a filter from a single word.
fn argumentless_filter(
    tokens: &mut TokenCursor,
    keyword: &str,
    filter: Filter,
) -> ParseResult<Filter> {
    take_literal(tokens, keyword)?;
    Ok(filter)
}

/// Returns a filter from a single word with an argument.
fn one_argument_filter<F: Fn(Word) -> Filter>(
    tokens: &mut TokenCursor,
    keyword: &str,
    func: F,
) -> ParseResult<Filter> {
    take_literal(tokens, keyword)?;
    let word = parse_word(tokens)?;
    Ok(func(word))
}

#[cfg(test)]
mod tests {
    use crate::{
        token::{Token, TokenContents},
        Span,
    };

    use super::*;

    fn parse(contents: Vec<TokenContents>) -> ParseResult<Filter> {
        let span = Span::new(0, 0); // Insignificant during these tests.
        let tokens: Vec<Token> = contents.into_iter().map(|c| Token::new(c, span)).collect();
        let mut token_cursor = TokenCursor::from(tokens);
        parse_filter(&mut token_cursor)
    }

    #[test]
    fn it_parses_sort() {
        assert_eq!(
            parse(vec![TokenContents::Literal("sort".into())]),
            Ok(Filter::Sort)
        );
    }

    #[test]
    fn it_parses_reverse() {
        assert_eq!(
            parse(vec![TokenContents::Literal("reverse".into())]),
            Ok(Filter::Reverse)
        );
    }

    #[test]
    fn it_parses_unique() {
        assert_eq!(
            parse(vec![TokenContents::Literal("unique".into())]),
            Ok(Filter::Unique)
        );
    }

    #[test]
    fn it_parses_lowercase() {
        assert_eq!(
            parse(vec![TokenContents::Literal("lower".into())]),
            Ok(Filter::Lower)
        );
    }

    #[test]
    fn it_parses_uppercase() {
        assert_eq!(
            parse(vec![TokenContents::Literal("upper".into())]),
            Ok(Filter::Upper)
        );
    }

    #[test]
    fn it_parses_index() {
        assert_eq!(
            parse(vec![
                TokenContents::Literal("index".into()),
                TokenContents::Literal("1".into()),
            ]),
            Ok(Filter::Index(Word::Literal("1".into())))
        );
    }
}
