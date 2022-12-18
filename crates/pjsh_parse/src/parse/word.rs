use pjsh_ast::{InterpolationUnit, List, ValuePipeline, Word};

use crate::{
    token::{self, TokenContents},
    ParseError,
};

use super::{
    cursor::TokenCursor,
    filter::parse_filter,
    program::{parse_program, parse_subshell_program, parse_subshell_word},
    utils::{skip_newlines, take_token, unexpected_token},
    ParseResult,
};

pub(crate) fn parse_word(tokens: &mut TokenCursor) -> ParseResult<Word> {
    match &tokens.peek().contents {
        TokenContents::Literal(_) => {
            let next = tokens.next();
            if let TokenContents::Literal(literal) = next.contents {
                Ok(Word::Literal(literal))
            } else {
                Err(ParseError::UnexpectedToken(next))
            }
        }
        TokenContents::DollarOpenParen => parse_subshell_word(tokens),
        TokenContents::DollarOpenBrace => parse_value_pipeline(tokens),
        TokenContents::TripleQuote => parse_triple_quoted(tokens),
        TokenContents::Quote => parse_quoted(tokens),
        TokenContents::Interpolation(_) => parse_interpolation(tokens),
        TokenContents::ProcessSubstitutionStart => parse_process_substitution(tokens),
        TokenContents::Variable(_) => {
            let TokenContents::Variable(variable) = tokens.next().contents else {
                unreachable!("This should already be filtered.");
            };
            Ok(Word::Variable(variable))
        }

        TokenContents::Eof => Err(ParseError::UnexpectedEof),
        _ => Err(ParseError::UnexpectedToken(tokens.peek().clone())),
    }
}

/// Parses a list of words surrounded by square brackets.
pub(crate) fn parse_list(tokens: &mut TokenCursor) -> Result<List, ParseError> {
    take_token(tokens, &TokenContents::OpenBracket)?;

    let mut list = List::default();
    loop {
        match tokens.peek().contents {
            TokenContents::Eol => skip_newlines(tokens),
            TokenContents::Eof => return Err(ParseError::IncompleteSequence),
            TokenContents::CloseBracket => break,
            _ => {
                list.items.push(parse_word(tokens)?);
            }
        }
    }

    take_token(tokens, &TokenContents::CloseBracket)?;

    Ok(list)
}

/// Parses an interpolation consisting of multiple interpolation units.
fn parse_interpolation(tokens: &mut TokenCursor) -> ParseResult<Word> {
    let Some(TokenContents::Interpolation(units)) = tokens
        .next_if(|t| matches!(t.contents, TokenContents::Interpolation(_)))
        .map(|t| t.contents) else {
            return Err(unexpected_token(tokens));
        };

    let mut word_units = Vec::with_capacity(units.len());
    for unit in units {
        word_units.push(parse_interpolation_unit(unit)?);
    }
    Ok(Word::Interpolation(word_units))
}

/// Parses a single interpolation unit.
fn parse_interpolation_unit(
    unit: token::InterpolationUnit,
) -> Result<InterpolationUnit, ParseError> {
    match unit {
        token::InterpolationUnit::Literal(literal) => Ok(InterpolationUnit::Literal(literal)),
        token::InterpolationUnit::Unicode(ch) => Ok(InterpolationUnit::Unicode(ch)),
        token::InterpolationUnit::Variable(var) => Ok(InterpolationUnit::Variable(var)),
        token::InterpolationUnit::ValuePipeline(pipeline_tokens) => {
            match parse_value_pipeline(&mut TokenCursor::from(pipeline_tokens))? {
                Word::Variable(variable) => Ok(InterpolationUnit::Variable(variable)),
                Word::ValuePipeline(pipeline) => Ok(InterpolationUnit::ValuePipeline(*pipeline)),
                _ => unreachable!("All possible parsed values should be covered"),
            }
        }
        token::InterpolationUnit::Subshell(subshell_tokens) => {
            let subshell_program = parse_program(&mut TokenCursor::from(subshell_tokens))?;
            Ok(InterpolationUnit::Subshell(subshell_program))
        }
    }
}

/// Parses a process substitution.
fn parse_process_substitution(tokens: &mut TokenCursor) -> ParseResult<Word> {
    tokens.next();

    let program = parse_subshell_program(tokens)?;

    if tokens.next_if_eq(TokenContents::CloseParen).is_none() {
        return Err(ParseError::IncompleteSequence);
    }

    Ok(Word::ProcessSubstitution(program))
}

/// Parses a triple quoted word.
fn parse_triple_quoted(tokens: &mut TokenCursor) -> ParseResult<Word> {
    tokens.next();
    let mut quoted = String::new();
    loop {
        let token = tokens.next();
        match token.contents {
            TokenContents::TripleQuote => break,
            TokenContents::Quoted(string) => quoted.push_str(&string),
            TokenContents::Eof => return Err(ParseError::UnexpectedEof),
            _ => return Err(ParseError::UnexpectedToken(token)),
        }
    }

    let mut lines = quoted.trim_end().lines();
    let mut string = String::new();
    let mut indent: usize = 0;

    if !matches!(lines.next(), Some(first_line) if first_line.is_empty()) {
        return Err(ParseError::InvalidSyntax(
            "multiline strings must contain at least one line".to_owned(),
        ));
    }

    if let Some(line) = lines.next() {
        let trimmed = line.trim_start_matches(char::is_whitespace);
        indent = line.len() - trimmed.len();
        string.push_str(trimmed);
    }

    for line in lines {
        let prefix = &line[..indent];
        if let Some((i, ch)) = prefix.char_indices().find(|(_, ch)| !ch.is_whitespace()) {
            return Err(ParseError::InvalidSyntax(format!(
                "expected an indentation of {indent}, found {ch} at character {i}"
            )));
        }

        string.push('\n');
        string.push_str(&line[indent..]);
    }

    Ok(Word::Quoted(string))
}

/// Parses a quoted word.
fn parse_quoted(tokens: &mut TokenCursor) -> ParseResult<Word> {
    tokens.next();
    let mut quoted = String::new();
    loop {
        let token = tokens.next();
        match token.contents {
            TokenContents::Quote => break,
            TokenContents::Quoted(string) => quoted.push_str(&string),
            TokenContents::Eof => return Err(ParseError::UnexpectedEof),
            _ => return Err(ParseError::UnexpectedToken(token)),
        }
    }
    Ok(Word::Quoted(quoted))
}

/// Parses a value pipeline.
fn parse_value_pipeline(tokens: &mut TokenCursor) -> ParseResult<Word> {
    take_token(tokens, &TokenContents::DollarOpenBrace)?;

    let base_token = tokens.next();
    let base = match base_token.contents {
        TokenContents::Literal(literal) => literal,
        TokenContents::Eof => return Err(ParseError::IncompleteSequence),
        _ => return Err(ParseError::UnexpectedToken(base_token)),
    };

    // Value pipelines without any filters can be simplified into single variables.
    // This does, however, require the function to return values of type Word rather
    // than values of type ValuePipeline.
    if take_token(tokens, &TokenContents::CloseBrace).is_ok() {
        return Ok(Word::Variable(base));
    }

    tokens.newline_is_whitespace(true);
    let mut filters = Vec::new();

    loop {
        let next = tokens.next();
        match next.contents {
            TokenContents::Pipe => (),
            TokenContents::CloseBrace => break,
            TokenContents::Eof => return Err(ParseError::IncompleteSequence),
            _ => return Err(ParseError::UnexpectedToken(next)),
        }

        filters.push(parse_filter(tokens)?);
    }

    Ok(Word::ValuePipeline(Box::new(ValuePipeline {
        base,
        filters,
    })))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pjsh_ast::{AndOr, Command, Filter, Pipeline, PipelineSegment, Program, Statement};

    use crate::{parse::pipeline::parse_pipeline, token::Token, Span};

    use super::*;

    #[test]
    fn it_parses_words() {
        let mut tokens = TokenCursor::from(vec![
            Token::new(TokenContents::Literal("first".into()), Span::new(0, 5)),
            Token::new(TokenContents::Quote, Span::new(5, 6)),
            Token::new(TokenContents::Quoted("a b".into()), Span::new(6, 18)),
            Token::new(TokenContents::Quote, Span::new(18, 19)),
        ]);
        assert_eq!(parse_word(&mut tokens), Ok(Word::Literal("first".into())));
        assert_eq!(parse_word(&mut tokens), Ok(Word::Quoted("a b".into())));
        assert_eq!(parse_word(&mut tokens), Err(ParseError::UnexpectedEof));
    }

    #[test]
    #[rustfmt::skip]
    fn it_parses_multiline_words() {
        let span = Span::new(0, 0); // Does not matter during this test.
        let mut tokens = TokenCursor::from(vec![
            Token::new(TokenContents::TripleQuote, span),
            Token::new(TokenContents::Quoted("\n    line1\n    line2\n  ".into()), span),
            Token::new(TokenContents::TripleQuote, span),
        ]);
        assert_eq!(parse_word(&mut tokens), Ok(Word::Quoted("line1\nline2".into())));

        let mut tokens = TokenCursor::from(vec![
            Token::new(TokenContents::TripleQuote, span),
            Token::new(TokenContents::Quoted("\n  line1\n    line2\n  line3\n".into()), span),
            Token::new(TokenContents::TripleQuote, span),
        ]);
        assert_eq!(
            parse_word(&mut tokens),
            Ok(Word::Quoted("line1\n  line2\nline3".into()))
        );
    }

    #[test]
    fn it_parses_lists() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_list(&mut TokenCursor::from(vec![
                Token::new(TokenContents::OpenBracket, span),
                Token::new(TokenContents::Literal("a".into()), span),
                Token::new(TokenContents::Literal("b".into()), span),
                Token::new(TokenContents::Literal("c".into()), span),
                Token::new(TokenContents::CloseBracket, span),
            ])),
            Ok(List::from(vec![
                Word::Literal("a".into()),
                Word::Literal("b".into()),
                Word::Literal("c".into()),
            ]))
        );
    }

    #[test]
    fn it_parses_brace_wrapped_variables() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_value_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::DollarOpenBrace, span),
                Token::new(TokenContents::Literal("variable".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Word::Variable("variable".into()))
        );
    }

    #[test]
    fn it_parses_value_pipelines() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_value_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::DollarOpenBrace, span),
                Token::new(TokenContents::Literal("base".into()), span),
                Token::new(TokenContents::Pipe, span),
                Token::new(TokenContents::Literal("sort".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Word::ValuePipeline(Box::new(ValuePipeline {
                base: "base".into(),
                filters: vec![Filter::Sort],
            })))
        );
    }

    #[test]
    fn parse_dollar_dollar() {
        assert_eq!(
            crate::parse("echo $$", &HashMap::new()),
            Ok(Program {
                statements: vec![Statement::AndOr(AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            arguments: vec![
                                Word::Literal("echo".into()),
                                Word::Variable("$".into())
                            ],
                            redirects: Vec::new(),
                        })]
                    }]
                })]
            })
        );
    }

    #[test]
    fn parse_process_substitution() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("cat".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::ProcessSubstitutionStart, span),
                Token::new(TokenContents::Literal("ls".into()), span),
                Token::new(TokenContents::CloseParen, span),
            ])),
            Ok(Pipeline {
                is_async: false,
                segments: vec![PipelineSegment::Command(Command {
                    arguments: vec![
                        Word::Literal("cat".into()),
                        Word::ProcessSubstitution(Program {
                            statements: vec![Statement::AndOr(AndOr {
                                operators: vec![],
                                pipelines: vec![Pipeline {
                                    is_async: false,
                                    segments: vec![PipelineSegment::Command(Command {
                                        arguments: vec![Word::Literal("ls".into())],
                                        redirects: Vec::new(),
                                    })]
                                }]
                            })]
                        }),
                    ],
                    redirects: Vec::new(),
                })]
            })
        );
    }
}
