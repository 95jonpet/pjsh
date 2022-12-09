use pjsh_ast::{Condition, Word};

use crate::{token::TokenContents, ParseError};

use super::{cursor::TokenCursor, utils::take_literal, word::parse_word, ParseResult};

/// Parses a condition.
pub(crate) fn parse_condition(tokens: &mut TokenCursor) -> ParseResult<Condition> {
    let mut lookahead = tokens.clone();
    lookahead
        .next_if_eq(TokenContents::DoubleOpenBracket)
        .ok_or_else(|| ParseError::UnexpectedToken(lookahead.peek().clone()))?;

    let inverted = take_literal(&mut lookahead, "!").is_ok();

    let mut condition = Result::Err(ParseError::InvalidSyntax("Unknown condition".into()))
        .or_else(|_| one_word_condition(&mut lookahead, "-n", Condition::NotEmpty))
        .or_else(|_| one_word_condition(&mut lookahead, "-z", Condition::Empty))
        .or_else(|_| one_word_condition(&mut lookahead, "-d", Condition::IsDirectory))
        .or_else(|_| one_word_condition(&mut lookahead, "is-dir", Condition::IsDirectory))
        .or_else(|_| one_word_condition(&mut lookahead, "-f", Condition::IsFile))
        .or_else(|_| one_word_condition(&mut lookahead, "is-file", Condition::IsFile))
        .or_else(|_| one_word_condition(&mut lookahead, "-e", Condition::IsPath))
        .or_else(|_| one_word_condition(&mut lookahead, "is-path", Condition::IsPath))
        .or_else(|_| two_word_condition(&mut lookahead, "==", Condition::Eq))
        .or_else(|_| two_word_condition(&mut lookahead, "!=", Condition::Ne))
        .or_else(|_| Ok(Condition::NotEmpty(parse_word(&mut lookahead)?)))?;

    lookahead
        .next_if_eq(TokenContents::DoubleCloseBracket)
        .ok_or_else(|| ParseError::UnexpectedToken(lookahead.peek().clone()))?;

    if inverted {
        condition = Condition::Invert(Box::new(condition));
    }

    *tokens = lookahead;
    Ok(condition)
}

/// Returns a condition from a single word.
///
/// Typically on the form `[[ keyword word ]]`.
fn one_word_condition<F: Fn(Word) -> Condition>(
    tokens: &mut TokenCursor,
    keyword: &str,
    func: F,
) -> ParseResult<Condition> {
    take_literal(tokens, keyword)?;
    let word = parse_word(tokens)?;
    Ok(func(word))
}

/// Returns a condition from two words.
///
/// Typically on the form `[[ a separator b ]]`.
fn two_word_condition<F: Fn(Word, Word) -> Condition>(
    tokens: &mut TokenCursor,
    separator: &str,
    func: F,
) -> ParseResult<Condition> {
    let mut inner_tokens = tokens.clone();
    let a = parse_word(&mut inner_tokens)?;
    take_literal(&mut inner_tokens, separator)?;
    let b = parse_word(&mut inner_tokens)?;
    *tokens = inner_tokens;
    Ok(func(a, b))
}

#[cfg(test)]
mod tests {
    use pjsh_ast::Word;

    use crate::{token::Token, Span};

    use super::*;

    fn parse(contents: Vec<TokenContents>) -> ParseResult<Condition> {
        let span = Span::new(0, 0); // Insignificant during these tests.
        let tokens: Vec<Token> = contents.into_iter().map(|c| Token::new(c, span)).collect();
        let mut token_cursor = TokenCursor::from(tokens);
        parse_condition(&mut token_cursor)
    }

    #[test]
    fn it_parses_is_dir() {
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("is-dir".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("path".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::IsDirectory(Word::Literal("path".into())))
        );
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("-d".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("path".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::IsDirectory(Word::Literal("path".into())))
        );
    }

    #[test]
    fn it_parses_is_file() {
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("is-file".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("path".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::IsFile(Word::Literal("path".into())))
        );
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("-f".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("path".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::IsFile(Word::Literal("path".into())))
        );
    }

    #[test]
    fn it_parses_is_path() {
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("is-path".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("path".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::IsPath(Word::Literal("path".into())))
        );
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("-e".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("path".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::IsPath(Word::Literal("path".into())))
        );
    }

    #[test]
    fn it_parses_eq() {
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("a".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("==".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("b".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::Eq(
                Word::Literal("a".into()),
                Word::Literal("b".into())
            ))
        );
    }

    #[test]
    fn it_parses_ne() {
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("a".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("!=".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("b".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::Ne(
                Word::Literal("a".into()),
                Word::Literal("b".into())
            ))
        );
    }

    #[test]
    fn it_parses_empty() {
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("-z".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("word".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::Empty(Word::Literal("word".into())))
        );
    }

    #[test]
    fn it_parses_not_empty() {
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("-n".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("word".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::NotEmpty(Word::Literal("word".into())))
        );
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("word".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::NotEmpty(Word::Literal("word".into())))
        );
    }

    #[test]
    fn it_parses_inverted_condition() {
        assert_eq!(
            parse(vec![
                TokenContents::DoubleOpenBracket,
                TokenContents::Literal("!".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("is-path".into()),
                TokenContents::Whitespace,
                TokenContents::Literal("path".into()),
                TokenContents::DoubleCloseBracket,
            ]),
            Ok(Condition::Invert(Box::new(Condition::IsPath(
                Word::Literal("path".into())
            ))))
        );
    }
}
