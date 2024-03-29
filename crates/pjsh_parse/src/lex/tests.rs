use std::collections::HashMap;

use crate::lex::input::Span;
use crate::token::{
    InterpolationUnit, Token,
    TokenContents::{self, *},
};

use super::lexer::*;

#[test]
fn lex_operators() {
    assert_eq!(tokens(":="), vec![Token::new(Assign, Span::new(0, 2))]);
    assert_eq!(
        tokens("::="),
        vec![Token::new(AssignResult, Span::new(0, 3))]
    );
    assert_eq!(tokens("&"), vec![Token::new(Amp, Span::new(0, 1))]);
    assert_eq!(tokens("|"), vec![Token::new(Pipe, Span::new(0, 1))]);
    assert_eq!(tokens(";"), vec![Token::new(Semi, Span::new(0, 1))]);
    assert_eq!(tokens("..."), vec![Token::new(Spread, Span::new(0, 3))]);

    assert_eq!(tokens("<"), vec![Token::new(FdReadTo(0), Span::new(0, 1))]);
    assert_eq!(
        tokens(">"),
        vec![Token::new(FdWriteFrom(1), Span::new(0, 1))]
    );
    assert_eq!(
        tokens(">>"),
        vec![Token::new(FdAppendFrom(1), Span::new(0, 2))]
    );

    assert_eq!(tokens("&&"), vec![Token::new(AndIf, Span::new(0, 2))]);
    assert_eq!(tokens("||"), vec![Token::new(OrIf, Span::new(0, 2))]);
}

#[test]
fn lex_eol() {
    assert_eq!(tokens("\n"), vec![Token::new(Eol, Span::new(0, 1))]);
    assert_eq!(tokens("\r"), vec![Token::new(Eol, Span::new(0, 1))]);
    // \r\n is considered a single char.
    assert_eq!(tokens("\r\n"), vec![Token::new(Eol, Span::new(0, 2))]);

    assert_eq!(
        tokens("\n\n"),
        vec![
            Token::new(Eol, Span::new(0, 1)),
            Token::new(Eol, Span::new(1, 2))
        ]
    );
}

#[test]
fn lex_comment() {
    assert_eq!(
        tokens("# This is a comment"),
        vec![Token::new(Comment, Span::new(0, 19))]
    );
}

#[test]
fn lex_surrounding_chars() {
    assert_eq!(
        tokens("$("),
        vec![Token::new(DollarOpenParen, Span::new(0, 2))]
    );
    assert_eq!(tokens("("), vec![Token::new(OpenParen, Span::new(0, 1))]);
    assert_eq!(tokens(")"), vec![Token::new(CloseParen, Span::new(0, 1))]);
    assert_eq!(
        tokens("${"),
        vec![Token::new(DollarOpenBrace, Span::new(0, 2))]
    );
    assert_eq!(tokens("{"), vec![Token::new(OpenBrace, Span::new(0, 1))]);
    assert_eq!(tokens("}"), vec![Token::new(CloseBrace, Span::new(0, 1))]);
    assert_eq!(tokens("["), vec![Token::new(OpenBracket, Span::new(0, 1))]);
    assert_eq!(tokens("]"), vec![Token::new(CloseBracket, Span::new(0, 1))]);
    assert_eq!(
        tokens("[["),
        vec![Token::new(DoubleOpenBracket, Span::new(0, 2))]
    );
    assert_eq!(
        tokens("]]"),
        vec![Token::new(DoubleCloseBracket, Span::new(0, 2))]
    );
}

#[test]
fn lex_literal() {
    assert_eq!(
        tokens("literal"),
        vec![Token::new(Literal("literal".into()), Span::new(0, 7))]
    );
    assert_eq!(
        tokens("lit123"),
        vec![Token::new(Literal("lit123".into()), Span::new(0, 6))]
    );
    assert_eq!(
        tokens("-lah"),
        vec![Token::new(Literal("-lah".into()), Span::new(0, 4))]
    );
}

#[test]
fn lex_variable() {
    assert_eq!(
        tokens("$variable"),
        vec![Token::new(Variable("variable".into()), Span::new(0, 9))]
    );
    assert_eq!(
        tokens("$my_var"),
        vec![Token::new(Variable("my_var".into()), Span::new(0, 7))]
    );
    assert_eq!(
        tokens("$_my_var"),
        vec![Token::new(Variable("_my_var".into()), Span::new(0, 8))]
    );
    assert_eq!(
        tokens("$?"),
        vec![Token::new(Variable("?".into()), Span::new(0, 2))]
    );
    assert_eq!(
        tokens("$0"),
        vec![Token::new(Variable("0".into()), Span::new(0, 2))]
    );
}

#[test]
fn lex_process_substitution() {
    assert_eq!(
        tokens("<(ls)"),
        vec![
            Token::new(ProcessSubstitutionStart, Span::new(0, 2)),
            Token::new(Literal("ls".into()), Span::new(2, 4)),
            Token::new(CloseParen, Span::new(4, 5)),
        ]
    );
}

#[test]
fn lex_interpolation_token() {
    assert_eq!(
        tokens("`literal $variable literal`"),
        vec![Token::new(
            Interpolation(vec![
                InterpolationUnit::Literal("literal ".into()),
                InterpolationUnit::Variable("variable".into()),
                InterpolationUnit::Literal(" literal".into()),
            ]),
            Span::new(0, 27)
        )]
    );
}

#[test]
fn lex_quoted_double() {
    assert_eq!(
        tokens(r#""quoted value""#),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("quoted value".into()), Span::new(1, 13)),
            Token::new(Quote, Span::new(13, 14))
        ]
    );
    assert_eq!(
        tokens("\"multiple\nlines\""),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("multiple\nlines".into()), Span::new(1, 15)),
            Token::new(Quote, Span::new(15, 16)),
        ]
    );
    assert_eq!(
        tokens(r#""C:\Dev""#),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted(r#"C:"#.into()), Span::new(1, 3)),
            Token::new(Quoted(r#"\"#.into()), Span::new(3, 4)),
            Token::new(Quoted(r#"Dev"#.into()), Span::new(4, 7)),
            Token::new(Quote, Span::new(7, 8))
        ]
    );
    assert_eq!(
        tokens(r#""escaped\"quote""#),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("escaped".into()), Span::new(1, 8)),
            Token::new(Quoted("\"".into()), Span::new(8, 10)), // Spans two chars of input.
            Token::new(Quoted("quote".into()), Span::new(10, 15)),
            Token::new(Quote, Span::new(15, 16))
        ]
    );

    assert_eq!(
        lex(r#""unterminated"#, &HashMap::new()),
        Err(LexError::UnexpectedEof)
    );
}

#[test]
fn lex_quoted_single() {
    assert_eq!(
        tokens("'quoted value'"),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("quoted value".into()), Span::new(1, 13)),
            Token::new(Quote, Span::new(13, 14))
        ]
    );
    assert_eq!(
        tokens("'multiple\nlines'"),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("multiple\nlines".into()), Span::new(1, 15)),
            Token::new(Quote, Span::new(15, 16)),
        ]
    );

    assert_eq!(
        lex("'unterminated", &HashMap::new()),
        Err(LexError::UnexpectedEof)
    );
    assert_eq!(
        lex(r#"'invalid end""#, &HashMap::new()),
        Err(LexError::UnexpectedEof)
    );
}

#[test]
fn lex_quoted_multiline_single() {
    assert_eq!(
        tokens("'''line1\nline2'''"),
        vec![
            Token::new(TripleQuote, Span::new(0, 3)),
            Token::new(Quoted("line1\nline2".into()), Span::new(3, 14)),
            Token::new(TripleQuote, Span::new(14, 17))
        ]
    );
    assert_eq!(
        tokens("'''first'second'third'''"),
        vec![
            Token::new(TripleQuote, Span::new(0, 3)),
            Token::new(Quoted("first'second'third".into()), Span::new(3, 21)),
            Token::new(TripleQuote, Span::new(21, 24))
        ]
    );
    assert_eq!(
        tokens(r#"'''\u{0020}'''"#),
        vec![
            Token::new(TripleQuote, Span::new(0, 3)),
            Token::new(Quoted(r#"\u{0020}"#.into()), Span::new(3, 11)),
            Token::new(TripleQuote, Span::new(11, 14))
        ]
    );
}

#[test]
fn lex_whitespace() {
    assert_eq!(tokens(" "), vec![Token::new(Whitespace, Span::new(0, 1))]);
    assert_eq!(tokens("  "), vec![Token::new(Whitespace, Span::new(0, 2))]);
    assert_eq!(tokens("\t"), vec![Token::new(Whitespace, Span::new(0, 1))]);
    assert_eq!(tokens("\t "), vec![Token::new(Whitespace, Span::new(0, 2))]);
}

#[test]
fn lex_word_interpolation() {
    assert_eq!(
        crate::lex_interpolation(r#"prompt [$PWD] \$"#)
            .unwrap()
            .contents,
        Interpolation(vec![
            InterpolationUnit::Literal("prompt [".into()),
            InterpolationUnit::Variable("PWD".into()),
            InterpolationUnit::Literal("] ".into()),
            InterpolationUnit::Literal("$".into())
        ])
    );
    assert_eq!(
        crate::lex_interpolation(r#"\e"#).unwrap().contents,
        Interpolation(vec![InterpolationUnit::Unicode('\u{001b}')])
    );
    assert_eq!(
        crate::lex_interpolation(r#"$(ls)"#).unwrap().contents,
        Interpolation(vec![InterpolationUnit::Subshell(vec![Token {
            contents: Literal("ls".into()),
            span: Span::new(2, 4)
        }])])
    );
    assert_eq!(
        crate::lex_interpolation(r#"$0"#).unwrap().contents,
        Interpolation(vec![InterpolationUnit::Variable("0".into())])
    );
    assert_eq!(
        crate::lex_interpolation(r#"$$"#).unwrap().contents,
        Interpolation(vec![InterpolationUnit::Variable("$".into())])
    );
}

#[test]
fn lex_incomplete_word_interpolation() {
    assert_eq!(
        crate::lex_interpolation(r#"$("#),
        Err(LexError::UnexpectedEof)
    );
    assert_eq!(
        crate::lex_interpolation(r#"${"#),
        Err(LexError::UnexpectedEof)
    );
}

#[test]
fn lex_interpolation_with_braces() {
    assert_eq!(
        lex(r#"`${var | len}`"#, &HashMap::default()),
        Ok(vec![Token::new(
            TokenContents::Interpolation(vec![InterpolationUnit::ValuePipeline(vec![
                Token::new(TokenContents::DollarOpenBrace, Span::new(1, 3)),
                Token::new(TokenContents::Literal("var".into()), Span::new(3, 6)),
                Token::new(TokenContents::Whitespace, Span::new(6, 7)),
                Token::new(TokenContents::Pipe, Span::new(7, 8)),
                Token::new(TokenContents::Whitespace, Span::new(8, 9)),
                Token::new(TokenContents::Literal("len".into()), Span::new(9, 12)),
                Token::new(TokenContents::CloseBrace, Span::new(12, 13)),
            ])]),
            Span::new(0, 14),
        ),])
    );
    assert_eq!(
        lex(r#"`${var|cmd arg}`"#, &HashMap::default()),
        Ok(vec![Token::new(
            TokenContents::Interpolation(vec![InterpolationUnit::ValuePipeline(vec![
                Token::new(TokenContents::DollarOpenBrace, Span::new(1, 3)),
                Token::new(TokenContents::Literal("var".into()), Span::new(3, 6)),
                Token::new(TokenContents::Pipe, Span::new(6, 7)),
                Token::new(TokenContents::Literal("cmd".into()), Span::new(7, 10)),
                Token::new(TokenContents::Whitespace, Span::new(10, 11)),
                Token::new(TokenContents::Literal("arg".into()), Span::new(11, 14)),
                Token::new(TokenContents::CloseBrace, Span::new(14, 15)),
            ])]),
            Span::new(0, 16),
        ),])
    );
}

fn tokens(src: &str) -> Vec<Token> {
    match lex(src, &HashMap::new()) {
        Ok(tokens) => tokens,
        Err(error) => panic!("Lexing failed: {}", error),
    }
}
