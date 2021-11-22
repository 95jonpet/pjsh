use pjsh_ast::{AndOr, AndOrOp, Assignment, Command, Pipeline, PipelineSegment, Statement, Word};

use super::parser::*;
use crate::{
    lex::lexer::{Span, Token},
    tokens::TokenContents::*,
};

#[test]
fn parse_word() {
    let mut parser = Parser::new(vec![
        Token::new(Literal("first"), Span::new(0, 5)),
        Token::new(Quote, Span::new(5, 6)),
        Token::new(Quoted("second third"), Span::new(6, 18)),
        Token::new(Quote, Span::new(18, 19)),
    ]);
    assert_eq!(parser.parse_word(), Ok(Word::Literal("first")));
    assert_eq!(parser.parse_word(), Ok(Word::Quoted("second third".into())));
    assert_eq!(parser.parse_word(), Err(ParseError::UnexpectedEof));
}

#[test]
fn parse_multiline_word() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(TripleQuote, span),
        Token::new(Quoted("\n    line1\n    line2\n  "), span),
        Token::new(TripleQuote, span),
    ]);
    assert_eq!(parser.parse_word(), Ok(Word::Quoted("line1\nline2".into())));

    let mut parser = Parser::new(vec![
        Token::new(TripleQuote, span),
        Token::new(Quoted("\n  line1\n    line2\n  line3\n"), span),
        Token::new(TripleQuote, span),
    ]);
    assert_eq!(
        parser.parse_word(),
        Ok(Word::Quoted("line1\n  line2\nline3".into()))
    );
}

#[test]
fn parse_and_or_andif() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(Literal("first"), span),
        Token::new(AndIf, span),
        Token::new(Literal("second"), span),
    ]);
    assert_eq!(
        parser.parse_and_or(),
        Ok(AndOr {
            operators: vec![AndOrOp::And],
            pipelines: vec![
                Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment {
                        command: Command {
                            program: Word::Literal("first"),
                            arguments: Vec::new(),
                            redirects: Vec::new(),
                        }
                    },]
                },
                Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment {
                        command: Command {
                            program: Word::Literal("second"),
                            arguments: Vec::new(),
                            redirects: Vec::new(),
                        }
                    },]
                }
            ]
        })
    );
}

#[test]
fn parse_and_or_orif() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(Literal("first"), span),
        Token::new(OrIf, span),
        Token::new(Literal("second"), span),
    ]);
    assert_eq!(
        parser.parse_and_or(),
        Ok(AndOr {
            operators: vec![AndOrOp::Or],
            pipelines: vec![
                Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment {
                        command: Command {
                            program: Word::Literal("first"),
                            arguments: Vec::new(),
                            redirects: Vec::new(),
                        }
                    },]
                },
                Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment {
                        command: Command {
                            program: Word::Literal("second"),
                            arguments: Vec::new(),
                            redirects: Vec::new(),
                        }
                    },]
                }
            ]
        })
    );
}

#[test]
fn parse_legacy_pipeline() {
    let mut parser = Parser::new(vec![
        Token::new(Literal("first"), Span::new(0, 5)),
        Token::new(Literal("second"), Span::new(6, 12)),
        Token::new(Pipe, Span::new(13, 14)),
        Token::new(Literal("third"), Span::new(15, 20)),
    ]);
    assert_eq!(
        parser.parse_pipeline(),
        Ok(Pipeline {
            is_async: false,
            segments: vec![
                PipelineSegment {
                    command: Command {
                        program: Word::Literal("first"),
                        arguments: vec![Word::Literal("second")],
                        redirects: Vec::new(),
                    }
                },
                PipelineSegment {
                    command: Command {
                        program: Word::Literal("third"),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    }
                }
            ]
        })
    );
}

#[test]
fn parse_legacy_pipeline_async() {
    let mut parser = Parser::new(vec![
        Token::new(Literal("command"), Span::new(0, 7)),
        Token::new(Amp, Span::new(7, 8)),
    ]);
    assert_eq!(
        parser.parse_pipeline(),
        Ok(Pipeline {
            is_async: true,
            segments: vec![PipelineSegment {
                command: Command {
                    program: Word::Literal("command"),
                    arguments: Vec::new(),
                    redirects: Vec::new(),
                }
            },]
        })
    );
}

#[test]
fn parse_smart_pipeline() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(PipeStart, span),
        Token::new(Literal("cmd1"), span),
        Token::new(Eol, span),
        Token::new(Pipe, span),
        Token::new(Literal("cmd2"), span),
        Token::new(Eol, span),
        Token::new(Semi, span),
    ]);
    assert_eq!(
        parser.parse_pipeline(),
        Ok(Pipeline {
            is_async: false,
            segments: vec![
                PipelineSegment {
                    command: Command {
                        program: Word::Literal("cmd1"),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    }
                },
                PipelineSegment {
                    command: Command {
                        program: Word::Literal("cmd2"),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    }
                }
            ]
        })
    );
}

#[test]
fn parse_smart_async_pipeline() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(PipeStart, span),
        Token::new(Literal("command"), span),
        Token::new(Amp, span), // Should mark the end of the pipeline.
    ]);
    assert_eq!(
        parser.parse_pipeline(),
        Ok(Pipeline {
            is_async: true,
            segments: vec![PipelineSegment {
                command: Command {
                    program: Word::Literal("command"),
                    arguments: Vec::new(),
                    redirects: Vec::new(),
                }
            },]
        })
    );
}

#[test]
fn parse_assignment_statement() {
    let mut parser = Parser::new(vec![
        Token::new(Literal("key"), Span::new(0, 3)),
        Token::new(Assign, Span::new(4, 5)),
        Token::new(Literal("value"), Span::new(6, 11)),
    ]);
    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Assignment(Assignment {
            key: Word::Literal("key"),
            value: Word::Literal("value"),
        }))
    )
}
