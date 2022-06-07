use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Command, ConditionalChain, ConditionalLoop, FileDescriptor,
    Function, InterpolationUnit, Pipeline, PipelineSegment, Program, Redirect, RedirectOperator,
    Statement, Word,
};

use super::parser::*;
use crate::{
    lex::lexer::{Span, Token},
    tokens::TokenContents::*,
    ParseError,
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
fn parse_condition() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(DoubleOpenBracket, span),
        Token::new(Literal("a".into()), span),
        Token::new(Literal("=".into()), span),
        Token::new(Literal("b".into()), span),
        Token::new(DoubleCloseBracket, span),
    ]);
    assert_eq!(
        parser.parse_and_or(),
        Ok(AndOr {
            operators: Vec::new(),
            pipelines: vec![Pipeline {
                is_async: false,
                segments: vec![PipelineSegment::Condition(vec![
                    Word::Literal("a".into()),
                    Word::Literal("=".into()),
                    Word::Literal("b".into()),
                ])]
            },]
        })
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
                    segments: vec![PipelineSegment::Command(Command {
                        program: Word::Literal("first".into()),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    }),]
                },
                Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment::Command(Command {
                        program: Word::Literal("second".into()),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    })]
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
                    segments: vec![PipelineSegment::Command(Command {
                        program: Word::Literal("first".into()),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    }),]
                },
                Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment::Command(Command {
                        program: Word::Literal("second".into()),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    }),]
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
                PipelineSegment::Command(Command {
                    program: Word::Literal("first".into()),
                    arguments: vec![Word::Literal("second".into())],
                    redirects: Vec::new(),
                }),
                PipelineSegment::Command(Command {
                    program: Word::Literal("third".into()),
                    arguments: Vec::new(),
                    redirects: Vec::new(),
                }),
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
            segments: vec![PipelineSegment::Command(Command {
                program: Word::Literal("command".into()),
                arguments: Vec::new(),
                redirects: Vec::new(),
            }),]
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
                PipelineSegment::Command(Command {
                    program: Word::Literal("cmd1".into()),
                    arguments: Vec::new(),
                    redirects: Vec::new(),
                }),
                PipelineSegment::Command(Command {
                    program: Word::Literal("cmd2".into()),
                    arguments: Vec::new(),
                    redirects: Vec::new(),
                }),
            ]
        })
    );
}

#[test]
fn parse_smart_pipeline_whitespace() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(PipeStart, span),
        Token::new(Literal("cmd".into()), span),
        Token::new(Eol, span),
        Token::new(Literal("arg1".into()), span),
        Token::new(Eol, span),
        Token::new(Literal("arg2".into()), span),
        Token::new(Eol, span),
        Token::new(Semi, span),
    ]);
    assert_eq!(
        parser.parse_pipeline(),
        Ok(Pipeline {
            is_async: false,
            segments: vec![PipelineSegment::Command(Command {
                program: Word::Literal("cmd".into()),
                arguments: vec![Word::Literal("arg1".into()), Word::Literal("arg2".into())],
                redirects: Vec::new(),
            }),]
        })
    );
}

#[test]
fn parse_smart_pipeline_partial() {
    let span = Span::new(0, 0); // Does not matter during this test.

    let mut tokens = vec![
        Token::new(PipeStart, span),
        Token::new(Whitespace, span),
        Token::new(Literal("cmd1".into()), span),
        Token::new(Eol, span),
    ];
    assert_eq!(
        Parser::new(tokens.clone()).parse_pipeline(),
        Err(ParseError::IncompleteSequence)
    );

    tokens.push(Token::new(Pipe, span));
    tokens.push(Token::new(Whitespace, span));
    tokens.push(Token::new(Literal("cmd2".into()), span));
    tokens.push(Token::new(Eol, span));
    assert_eq!(
        Parser::new(tokens.clone()).parse_pipeline(),
        Err(ParseError::IncompleteSequence)
    );

    tokens.push(Token::new(Semi, span));
    assert!(Parser::new(tokens.clone()).parse_pipeline().is_ok());
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
            segments: vec![PipelineSegment::Command(Command {
                program: Word::Literal("command".into()),
                arguments: Vec::new(),
                redirects: Vec::new(),
            }),]
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
fn parse_function_statement() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(Literal("fn".into()), span),
        Token::new(Literal("function_name".into()), span),
        Token::new(OpenParen, span),
        Token::new(Literal("arg".into()), span),
        Token::new(CloseParen, span),
        Token::new(OpenBrace, span),
        Token::new(Literal("echo".into()), span),
        Token::new(Literal("test".into()), span),
        Token::new(CloseBrace, span),
    ]);
    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::Function(Function {
            name: "function_name".into(),
            args: vec!["arg".into()].into(),
            body: Program {
                statements: vec![Statement::AndOr(AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            program: Word::Literal("echo".into()),
                            arguments: vec![Word::Literal("test".into())],
                            redirects: Vec::new(),
                        })]
                    }]
                })]
            }
        }))
    )
}

#[test]
fn parse_if_statement() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(Literal("if".into()), span),
        Token::new(Literal("true".into()), span),
        Token::new(OpenBrace, span),
        Token::new(Literal("echo".into()), span),
        Token::new(Literal("test".into()), span),
        Token::new(CloseBrace, span),
    ]);
    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::If(ConditionalChain {
            conditions: vec![AndOr {
                operators: Vec::new(),
                pipelines: vec![Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment::Command(Command {
                        program: Word::Literal("true".into()),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    })]
                }]
            }],
            branches: vec![Program {
                statements: vec![Statement::AndOr(AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            program: Word::Literal("echo".into()),
                            arguments: vec![Word::Literal("test".into())],
                            redirects: Vec::new(),
                        })]
                    }]
                })]
            }]
        }))
    )
}

#[test]
fn parse_if_statement_with_multiple_branches() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(Literal("if".into()), span),
        Token::new(Literal("false".into()), span),
        Token::new(OpenBrace, span),
        Token::new(Literal("echo".into()), span),
        Token::new(Literal("first".into()), span),
        Token::new(CloseBrace, span),
        Token::new(Literal("else".into()), span),
        Token::new(Literal("if".into()), span),
        Token::new(Literal("false".into()), span),
        Token::new(OpenBrace, span),
        Token::new(Literal("echo".into()), span),
        Token::new(Literal("second".into()), span),
        Token::new(CloseBrace, span),
        Token::new(Literal("else".into()), span),
        Token::new(OpenBrace, span),
        Token::new(Literal("echo".into()), span),
        Token::new(Literal("third".into()), span),
        Token::new(CloseBrace, span),
    ]);
    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::If(ConditionalChain {
            conditions: vec![
                AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            program: Word::Literal("false".into()),
                            arguments: Vec::new(),
                            redirects: Vec::new(),
                        })]
                    }]
                },
                AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            program: Word::Literal("false".into()),
                            arguments: Vec::new(),
                            redirects: Vec::new(),
                        })]
                    }]
                }
            ],
            branches: vec![
                Program {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                program: Word::Literal("echo".into()),
                                arguments: vec![Word::Literal("first".into())],
                                redirects: Vec::new(),
                            })]
                        }]
                    })]
                },
                Program {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                program: Word::Literal("echo".into()),
                                arguments: vec![Word::Literal("second".into())],
                                redirects: Vec::new(),
                            })]
                        }]
                    })]
                },
                Program {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                program: Word::Literal("echo".into()),
                                arguments: vec![Word::Literal("third".into())],
                                redirects: Vec::new(),
                            })]
                        }]
                    })]
                }
            ]
        }))
    )
}

#[test]
fn parse_while_loop() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(Literal("while".into()), span),
        Token::new(Literal("false".into()), span),
        Token::new(OpenBrace, span),
        Token::new(Literal("echo".into()), span),
        Token::new(Literal("test".into()), span),
        Token::new(CloseBrace, span),
    ]);
    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::While(ConditionalLoop {
            condition: AndOr {
                operators: Vec::new(),
                pipelines: vec![Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment::Command(Command {
                        program: Word::Literal("false".into()),
                        arguments: Vec::new(),
                        redirects: Vec::new(),
                    })]
                }]
            },
            body: Program {
                statements: vec![Statement::AndOr(AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            program: Word::Literal("echo".into()),
                            arguments: vec![Word::Literal("test".into())],
                            redirects: Vec::new(),
                        })]
                    }]
                })]
            }
        }))
    )
}

#[test]
fn parse_statement_before_unexpected() {
    let span = Span::new(0, 0); // Does not matter during this test.
    let mut parser = Parser::new(vec![
        Token::new(Literal("echo".into()), span),
        Token::new(Literal("test".into()), span),
        Token::new(CloseParen, span), // Unexpected token.
    ]);

    // First, return the valid statement.
    assert_eq!(
        parser.parse_statement(),
        Ok(Statement::AndOr(AndOr {
            operators: Vec::new(),
            pipelines: vec![Pipeline {
                is_async: false,
                segments: vec![PipelineSegment::Command(Command {
                    program: Word::Literal("echo".into()),
                    arguments: vec![Word::Literal("test".into())],
                    redirects: Vec::new(),
                })]
            }]
        }))
    );

    // Then, return the parse error.
    assert_eq!(
        parser.parse_statement(),
        Err(ParseError::UnexpectedToken(Token {
            contents: CloseParen,
            span
        }))
    );
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

#[test]
fn parse_program() {
    assert_eq!(
        crate::parse("cmd1 arg1 ; cmd2 arg2"),
        Ok(Program {
            statements: vec![
                Statement::AndOr(AndOr {
                    operators: vec![],
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            program: Word::Literal("cmd1".into()),
                            arguments: vec![Word::Literal("arg1".into())],
                            redirects: Vec::new(),
                        }),]
                    }]
                }),
                Statement::AndOr(AndOr {
                    operators: vec![],
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            program: Word::Literal("cmd2".into()),
                            arguments: vec![Word::Literal("arg2".into())],
                            redirects: Vec::new(),
                        }),]
                    }]
                })
            ]
        })
    );
}

#[test]
fn parse_subshell() {
    assert_eq!(
        crate::parse("(cmd1 arg1 ; cmd2 arg2)"),
        Ok(Program {
            statements: vec![Statement::Subshell(Program {
                statements: vec![
                    Statement::AndOr(AndOr {
                        operators: vec![],
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                program: Word::Literal("cmd1".into()),
                                arguments: vec![Word::Literal("arg1".into())],
                                redirects: Vec::new(),
                            }),]
                        }]
                    }),
                    Statement::AndOr(AndOr {
                        operators: vec![],
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                program: Word::Literal("cmd2".into()),
                                arguments: vec![Word::Literal("arg2".into())],
                                redirects: Vec::new(),
                            }),]
                        }]
                    })
                ]
            })]
        })
    );
}

#[test]
fn parse_subshell_interpolation() {
    assert_eq!(
        crate::parse("echo `today: $(date)`"),
        Ok(Program {
            statements: vec![Statement::AndOr(AndOr {
                operators: Vec::new(),
                pipelines: vec![Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment::Command(Command {
                        program: Word::Literal("echo".into()),
                        arguments: vec![Word::Interpolation(vec![
                            InterpolationUnit::Literal("today: ".into()),
                            InterpolationUnit::Subshell(Program {
                                statements: vec![Statement::AndOr(AndOr {
                                    operators: vec![],
                                    pipelines: vec![Pipeline {
                                        is_async: false,
                                        segments: vec![PipelineSegment::Command(Command {
                                            program: Word::Literal("date".into()),
                                            arguments: Vec::new(),
                                            redirects: Vec::new(),
                                        }),]
                                    }]
                                }),]
                            })
                        ])],
                        redirects: Vec::new(),
                    })]
                }]
            })]
        })
    );
}

#[test]
fn parse_dollar_dollar() {
    assert_eq!(
        crate::parse("echo $$"),
        Ok(Program {
            statements: vec![Statement::AndOr(AndOr {
                operators: Vec::new(),
                pipelines: vec![Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment::Command(Command {
                        program: Word::Literal("echo".into()),
                        arguments: vec![Word::Variable("$".into())],
                        redirects: Vec::new(),
                    })]
                }]
            })]
        })
    );
}
