use super::lexer::*;
use crate::tokens::{InterpolationUnit, TokenContents::*};

#[test]
fn lex_operators() {
    assert_eq!(tokens(":="), vec![Token::new(Assign, Span::new(0, 2))]);
    assert_eq!(tokens("&"), vec![Token::new(Amp, Span::new(0, 1))]);
    assert_eq!(tokens("|"), vec![Token::new(Pipe, Span::new(0, 1))]);
    assert_eq!(tokens(";"), vec![Token::new(Semi, Span::new(0, 1))]);

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
fn lex_literal() {
    assert_eq!(
        tokens("literal"),
        vec![Token::new(Literal("literal"), Span::new(0, 7))]
    );
    assert_eq!(
        tokens("lit123"),
        vec![Token::new(Literal("lit123"), Span::new(0, 6))]
    );
    assert_eq!(
        tokens("-lah"),
        vec![Token::new(Literal("-lah"), Span::new(0, 4))]
    );
}

#[test]
fn lex_variable() {
    assert_eq!(
        tokens("$variable"),
        vec![Token::new(Variable("variable"), Span::new(0, 9))]
    );
    assert_eq!(
        tokens("$my_var"),
        vec![Token::new(Variable("my_var"), Span::new(0, 7))]
    );
    assert_eq!(
        tokens("$_my_var"),
        vec![Token::new(Variable("_my_var"), Span::new(0, 8))]
    );
    assert_eq!(
        tokens("${wrapped_var}"),
        vec![Token::new(Variable("wrapped_var"), Span::new(0, 14))]
    );
}

#[test]
fn lex_interpolation_token() {
    assert_eq!(
        tokens(r#"$"literal $variable literal""#),
        vec![Token::new(
            Interpolation(vec![
                InterpolationUnit::Literal("literal "),
                InterpolationUnit::Variable("variable"),
                InterpolationUnit::Literal(" literal"),
            ]),
            Span::new(0, 28)
        )]
    );
}

#[test]
fn lex_quoted_double() {
    assert_eq!(
        tokens(r#""quoted value""#),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("quoted value"), Span::new(1, 13)),
            Token::new(Quote, Span::new(13, 14))
        ]
    );
    assert_eq!(
        tokens("\"multiple\nlines\""),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("multiple\nlines"), Span::new(1, 15)),
            Token::new(Quote, Span::new(15, 16)),
        ]
    );
    assert_eq!(
        tokens(r#""C:\Dev""#),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted(r#"C:"#), Span::new(1, 3)),
            Token::new(Quoted(r#"\"#), Span::new(3, 4)),
            Token::new(Quoted(r#"Dev"#), Span::new(4, 7)),
            Token::new(Quote, Span::new(7, 8))
        ]
    );
    assert_eq!(
        tokens(r#""escaped\"quote""#),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("escaped"), Span::new(1, 8)),
            Token::new(Quoted("\""), Span::new(8, 10)), // Spans two chars of input.
            Token::new(Quoted("quote"), Span::new(10, 15)),
            Token::new(Quote, Span::new(15, 16))
        ]
    );

    assert_eq!(lex(r#""unterminated"#), Err(LexError::UnexpectedEof));
}

#[test]
fn lex_quoted_single() {
    assert_eq!(
        tokens("'quoted value'"),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("quoted value"), Span::new(1, 13)),
            Token::new(Quote, Span::new(13, 14))
        ]
    );
    assert_eq!(
        tokens("'multiple\nlines'"),
        vec![
            Token::new(Quote, Span::new(0, 1)),
            Token::new(Quoted("multiple\nlines"), Span::new(1, 15)),
            Token::new(Quote, Span::new(15, 16)),
        ]
    );

    assert_eq!(lex("'unterminated"), Err(LexError::UnexpectedEof));
    assert_eq!(lex(r#"'invalid end""#), Err(LexError::UnexpectedEof));
}

#[test]
fn lex_quoted_multiline_single() {
    assert_eq!(
        tokens("'''line1\nline2'''"),
        vec![
            Token::new(TripleQuote, Span::new(0, 3)),
            Token::new(Quoted("line1\nline2"), Span::new(3, 14)),
            Token::new(TripleQuote, Span::new(14, 17))
        ]
    );
    assert_eq!(
        tokens("'''first'second'third'''"),
        vec![
            Token::new(TripleQuote, Span::new(0, 3)),
            Token::new(Quoted("first"), Span::new(3, 8)),
            Token::new(Quoted("'"), Span::new(8, 9)),
            Token::new(Quoted("second"), Span::new(9, 15)),
            Token::new(Quoted("'"), Span::new(15, 16)),
            Token::new(Quoted("third"), Span::new(16, 21)),
            Token::new(TripleQuote, Span::new(21, 24))
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
            InterpolationUnit::Literal("prompt ["),
            InterpolationUnit::Variable("PWD"),
            InterpolationUnit::Literal("] "),
            InterpolationUnit::Literal("$")
        ])
    );
    assert_eq!(
        crate::lex_interpolation(r#"\e"#).unwrap().contents,
        Interpolation(vec![InterpolationUnit::Unicode('\u{001b}')])
    );
}

fn tokens(src: &str) -> Vec<Token> {
    match lex(src) {
        Ok(tokens) => tokens,
        Err(error) => panic!("Lexing failed: {}", error),
    }
}
