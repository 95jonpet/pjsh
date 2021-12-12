use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Command, FileDescriptor, Pipeline, PipelineSegment, Redirect,
    RedirectOperator, Statement, Word,
};

use super::parser::*;
use crate::{
    lex::lexer::{Span, Token},
    tokens::TokenContents::*,
};

#[test]
fn parse_word() {
    let mut parser = Parser::new(vec![
        Token::new(Literal("first".into()), Span::new(0, 5)),
        Token::new(Quote, Span::new(5, 6)),
        Token::new(Quoted("second third".into()), Span::new(6, 18)),
        Token::new(Quote, Span::new(18, 19)),
    ]);
    assert_eq!(parser.parse_word(), Ok(Word::Literal("first".into())));
    assert_eq!(parser.parse_word(), Ok(Word::Quoted("second third".into())));
    assert_eq!(parser.parse_word(), Err(ParseError::UnexpectedEof));
}

#[test]
fn parse_multiline_word() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(TripleQuote, span),
        Token::new(Quoted("\n    line1\n    line2\n  ".into()), span),
        Token::new(TripleQuote, span),
    ]);
    assert_eq!(parser.parse_word(), Ok(Word::Quoted("line1\nline2".into())));

    let mut parser = Parser::new(vec![
        Token::new(TripleQuote, span),
        Token::new(Quoted("\n  line1\n    line2\n  line3\n".into()), span),
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
        Token::new(Literal("first".into()), span),
        Token::new(AndIf, span),
        Token::new(Literal("second".into()), span),
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
                            program: Word::Literal("first".into()),
                            arguments: Vec::new(),
                            redirects: Vec::new(),
                        }
                    },]
                },
                Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment {
                        command: Command {
                            program: Word::Literal("second".into()),
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
        Token::new(Literal("first".into()), span),
        Token::new(OrIf, span),
        Token::new(Literal("second".into()), span),
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
                            program: Word::Literal("first".into()),
                            arguments: Vec::new(),
                            redirects: Vec::new(),
                        }
                    },]
                },
                Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment {
                        command: Command {
                            program: Word::Literal("second".into()),
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
        Token::new(Literal("first".into()), Span::new(0, 5)),
        Token::new(Literal("second".into()), Span::new(6, 12)),
        Token::new(Pipe, Span::new(13, 14)),
        Token::new(Literal("third".into()), Span::new(15, 20)),
    ]);
    assert_eq!(
        parser.parse_pipeline(),
        Ok(Pipeline {
            is_async: false,
            segments: vec![
                PipelineSegment {
                    command: Command {
                        program: Word::Literal("first".into()),
                        arguments: vec![Word::Literal("second".into())],
                        redirects: Vec::new(),
                    }
                },
                PipelineSegment {
                    command: Command {
                        program: Word::Literal("third".into()),
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
        Token::new(Literal("command".into()), Span::new(0, 7)),
        Token::new(Amp, Span::new(7, 8)),
    ]);
    assert_eq!(
        parser.parse_pipeline(),
        Ok(Pipeline {
            is_async: true,
            segments: vec![PipelineSegment {
                command: Command {
                    program: Word::Literal("command".into()),
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
        Token::new(Literal("cmd1".into()), span),
        Token::new(Eol, span),
        Token::new(Pipe, span),
        Token::new(Literal("cmd2".into()), span),
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
                        program: Word::Literal("cmd1".into()),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    }
                },
                PipelineSegment {
                    command: Command {
                        program: Word::Literal("cmd2".into()),
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
        Token::new(Literal("command".into()), span),
        Token::new(Amp, span), // Should mark the end of the pipeline.
    ]);
    assert_eq!(
        parser.parse_pipeline(),
        Ok(Pipeline {
            is_async: true,
            segments: vec![PipelineSegment {
                command: Command {
                    program: Word::Literal("command".into()),
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
        Token::new(Literal("key".into()), Span::new(0, 3)),
        Token::new(Assign, Span::new(4, 5)),
        Token::new(Literal("value".into()), Span::new(6, 11)),
    ]);
    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Assignment(Assignment {
            key: Word::Literal("key".into()),
            value: Word::Literal("value".into()),
        }))
    )
}

#[test]
fn parse_redirect_read() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(FdReadTo(0), span),
        Token::new(Literal("file".into()), span),
    ]);
    assert_eq!(
        parser.parse_redirect(),
        Ok(Redirect {
            source: FileDescriptor::File(Word::Literal("file".into())),
            target: FileDescriptor::Number(0),
            operator: RedirectOperator::Write
        })
    )
}

#[test]
fn parse_redirect_write() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(FdWriteFrom(1), span),
        Token::new(Literal("file".into()), span),
    ]);
    assert_eq!(
        parser.parse_redirect(),
        Ok(Redirect {
            source: FileDescriptor::Number(1),
            target: FileDescriptor::File(Word::Literal("file".into())),
            operator: RedirectOperator::Write
        })
    )
}

#[test]
fn parse_redirect_append() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(FdAppendFrom(1), span),
        Token::new(Literal("file".into()), span),
    ]);
    assert_eq!(
        parser.parse_redirect(),
        Ok(Redirect {
            source: FileDescriptor::Number(1),
            target: FileDescriptor::File(Word::Literal("file".into())),
            operator: RedirectOperator::Append
        })
    )
}