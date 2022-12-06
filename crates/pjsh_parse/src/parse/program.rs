use pjsh_ast::{AndOr, AndOrOp, Program, Statement, Word};

use crate::{
    token::{Token, TokenContents},
    ParseError,
};

use super::{
    cursor::TokenCursor,
    pipeline::parse_pipeline,
    statement::parse_statement,
    utils::{take_token, unexpected_token},
    ParseResult,
};

/// Parses [`Program`] by consuming all remaining input.
pub fn parse_program(tokens: &mut TokenCursor) -> ParseResult<Program> {
    let mut program = Program::new();

    loop {
        match parse_statement(tokens) {
            // Fill the program while more statements can be parsed.
            Ok(statement) => {
                program.statement(statement);
            }

            // There is no more input, and no half-parsed statements.
            // Parsing is completed.
            Err(ParseError::UnexpectedEof) => break,

            // Incomplete sequences must be emitted as parse errors so that interactive shells
            // can request more input from the user.
            Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),

            // Generic errors are not recoverable and must be emitted.
            Err(error) => return Err(error),
        }
    }

    // All tokens should be consumed when parsing a valid program.
    if tokens.peek().contents != TokenContents::Eof {
        return Err(unexpected_token(tokens));
    }

    Ok(program)
}

pub(crate) fn parse_subshell_program(tokens: &mut TokenCursor) -> ParseResult<Program> {
    let mut subshell_program = Program::new();
    loop {
        match parse_statement(tokens) {
            Ok(statement) => {
                subshell_program.statement(statement);
            }
            Err(ParseError::UnexpectedToken(Token {
                contents: TokenContents::CloseParen,
                span: _,
            })) => {
                break;
            }
            Err(ParseError::UnexpectedEof) => {
                return Err(ParseError::IncompleteSequence);
            }
            Err(error) => {
                return Err(error);
            }
        }
    }

    Ok(subshell_program)
}

/// Parses a non-empty subshell.
///
/// Note that tokens are consumed as long as the subshell is opened - even if the subshell is
/// empty.
pub(crate) fn parse_subshell(tokens: &mut TokenCursor) -> ParseResult<Statement> {
    take_token(tokens, &TokenContents::OpenParen)?;

    let subshell_program = parse_subshell_program(tokens)?;

    // A subshell must be terminated by a closing parenthesis.
    take_token(tokens, &TokenContents::CloseParen)?;

    // A subshell must not be empty.
    if subshell_program.statements.is_empty() {
        return Err(ParseError::EmptySubshell);
    }

    Ok(Statement::Subshell(subshell_program))
}

/// Parses a non-empty subshell word.
///
/// Note that tokens are consumed as long as the subshell is opened - even if the subshell is
/// empty.
pub(crate) fn parse_subshell_word(tokens: &mut TokenCursor) -> ParseResult<Word> {
    take_token(tokens, &TokenContents::DollarOpenParen)?;

    let subshell_program = parse_subshell_program(tokens)?;

    // A subshell must be terminated by a closing parenthesis.
    take_token(tokens, &TokenContents::CloseParen)?;

    // A subshell must not be empty.
    if subshell_program.statements.is_empty() {
        return Err(ParseError::EmptySubshell);
    }

    Ok(Word::Subshell(subshell_program))
}

/// Parses an [`AndOr`] consisting of one or more [`Pipeline`] definitions.
pub fn parse_and_or(tokens: &mut TokenCursor) -> ParseResult<AndOr> {
    let mut and_or = AndOr::default();
    and_or.pipelines.push(parse_pipeline(tokens)?);

    loop {
        if tokens.next_if_eq(TokenContents::Eof).is_some() {
            break;
        }

        // Semi tokens terminate the current statement.
        if tokens.next_if_eq(TokenContents::Semi).is_some() {
            break;
        }

        let operator = match tokens.peek().contents {
            TokenContents::AndIf => AndOrOp::And,
            TokenContents::OrIf => AndOrOp::Or,
            _ => break,
        };
        tokens.next();

        and_or.operators.push(operator);
        and_or.pipelines.push(parse_pipeline(tokens)?);
    }

    Ok(and_or)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pjsh_ast::{Command, InterpolationUnit, Pipeline, PipelineSegment};

    use crate::{parse::program::parse_program, Span};

    use super::*;

    #[test]
    fn parse_and_or_andif() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_and_or(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("first".into()), span),
                Token::new(TokenContents::AndIf, span),
                Token::new(TokenContents::Literal("second".into()), span),
            ])),
            Ok(AndOr {
                operators: vec![AndOrOp::And],
                pipelines: vec![
                    Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            arguments: vec![Word::Literal("first".into())],
                            redirects: Vec::new(),
                        }),]
                    },
                    Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            arguments: vec![Word::Literal("second".into())],
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
        assert_eq!(
            parse_and_or(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("first".into()), span),
                Token::new(TokenContents::OrIf, span),
                Token::new(TokenContents::Literal("second".into()), span),
            ])),
            Ok(AndOr {
                operators: vec![AndOrOp::Or],
                pipelines: vec![
                    Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            arguments: vec![Word::Literal("first".into())],
                            redirects: Vec::new(),
                        }),]
                    },
                    Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            arguments: vec![Word::Literal("second".into())],
                            redirects: Vec::new(),
                        }),]
                    }
                ]
            })
        );
    }

    #[test]
    fn it_parses_programs() {
        assert_eq!(
            crate::parse("cmd1 arg1 ; cmd2 arg2", &HashMap::new()),
            Ok(Program {
                statements: vec![
                    Statement::AndOr(AndOr {
                        operators: vec![],
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![
                                    Word::Literal("cmd1".into()),
                                    Word::Literal("arg1".into())
                                ],
                                redirects: Vec::new(),
                            }),]
                        }]
                    }),
                    Statement::AndOr(AndOr {
                        operators: vec![],
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![
                                    Word::Literal("cmd2".into()),
                                    Word::Literal("arg2".into())
                                ],
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
            crate::parse("(cmd1 arg1 ; cmd2 arg2)", &HashMap::new()),
            Ok(Program {
                statements: vec![Statement::Subshell(Program {
                    statements: vec![
                        Statement::AndOr(AndOr {
                            operators: vec![],
                            pipelines: vec![Pipeline {
                                is_async: false,
                                segments: vec![PipelineSegment::Command(Command {
                                    arguments: vec![
                                        Word::Literal("cmd1".into()),
                                        Word::Literal("arg1".into())
                                    ],
                                    redirects: Vec::new(),
                                }),]
                            }]
                        }),
                        Statement::AndOr(AndOr {
                            operators: vec![],
                            pipelines: vec![Pipeline {
                                is_async: false,
                                segments: vec![PipelineSegment::Command(Command {
                                    arguments: vec![
                                        Word::Literal("cmd2".into()),
                                        Word::Literal("arg2".into())
                                    ],
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
    fn parse_incomplete_subshell() {
        assert_eq!(
            parse_program(&mut TokenCursor::from(vec![
                Token::new(TokenContents::OpenParen, Span::new(0, 1)),
                Token::new(TokenContents::Literal("true".into()), Span::new(1, 5)),
            ])),
            Err(ParseError::IncompleteSequence)
        );
    }

    #[test]
    fn parse_subshell_over_multiple_lines() {
        assert_eq!(
            crate::parse("(\ncmd arg\n)", &HashMap::new()),
            Ok(Program {
                statements: vec![Statement::Subshell(Program {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: vec![],
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![
                                    Word::Literal("cmd".into()),
                                    Word::Literal("arg".into())
                                ],
                                redirects: Vec::new(),
                            }),]
                        }]
                    }),]
                })]
            })
        );
    }

    #[test]
    fn parse_subshell_interpolation() {
        assert_eq!(
            crate::parse("echo `today: $(date)`", &HashMap::new()),
            Ok(Program {
                statements: vec![Statement::AndOr(AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            arguments: vec![
                                Word::Literal("echo".into()),
                                Word::Interpolation(vec![
                                    InterpolationUnit::Literal("today: ".into()),
                                    InterpolationUnit::Subshell(Program {
                                        statements: vec![Statement::AndOr(AndOr {
                                            operators: vec![],
                                            pipelines: vec![Pipeline {
                                                is_async: false,
                                                segments: vec![PipelineSegment::Command(Command {
                                                    arguments: vec![Word::Literal("date".into())],
                                                    redirects: Vec::new(),
                                                }),]
                                            }]
                                        }),]
                                    })
                                ])
                            ],
                            redirects: Vec::new(),
                        })]
                    }]
                })]
            })
        );
    }
}
