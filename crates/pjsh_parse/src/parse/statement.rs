use pjsh_ast::{
    Assignment, Block, ConditionalChain, ConditionalLoop, ForIterableLoop, ForOfIterableLoop,
    Function, Iterable, Statement, Word,
};

use crate::{parse::word::parse_word, token::TokenContents, ParseError};

use super::{
    cursor::TokenCursor,
    iterable::{iteration_rule, parse_iterable},
    program::{parse_and_or, parse_subshell},
    utils::{skip_newlines, take_literal, take_token, unexpected_token},
    word::parse_list,
    ParseResult,
};

/// Tries to parse a [`Statement`] from the next tokens of input.
pub(crate) fn parse_statement(tokens: &mut TokenCursor) -> ParseResult<Statement> {
    tokens.newline_is_whitespace(false); // Ensure clean start.
    skip_newlines(tokens);

    // Try to parse a subshell.
    match parse_subshell(tokens) {
        Ok(subshell_statement) => return Ok(subshell_statement),
        Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
        _ => (),
    }

    // Try to parse an if-statement.
    match parse_if_statement(tokens) {
        Ok(statement) => return Ok(statement),
        Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
        _ => (),
    }

    // Try to parse a for-loops.
    match parse_for_loop(tokens) {
        Ok(statement) => return Ok(statement),
        Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
        _ => (),
    }

    // Try to parse a while-loop.
    match parse_while_loop(tokens) {
        Ok(statement) => return Ok(statement),
        Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
        _ => (),
    }

    // Try to parse a function declaration.
    match parse_function(tokens) {
        Ok(function_statement) => return Ok(function_statement),
        Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
        _ => (),
    }

    // Try to parse an assignment.
    let mut assignment_iter = tokens.clone();
    assignment_iter.next();
    if assignment_iter.peek().contents == TokenContents::Assign {
        let key = parse_word(tokens)?;

        debug_assert_eq!(tokens.peek().contents, TokenContents::Assign);
        tokens.next();

        let value = parse_word(tokens)?;
        return Ok(Statement::Assignment(Assignment { key, value }));
    }

    Ok(Statement::AndOr(parse_and_or(tokens)?))
}

/// Parses a function declaration,
fn parse_function(tokens: &mut TokenCursor) -> ParseResult<Statement> {
    take_literal(tokens, "fn")?;

    match tokens.next().contents {
        TokenContents::Literal(name) => {
            take_token(tokens, &TokenContents::OpenParen)?;

            // Parse argument list.
            let mut args = Vec::new();
            while let Some(token) =
                tokens.next_if(|t| matches!(&t.contents, &TokenContents::Literal(_)))
            {
                match token.contents {
                    TokenContents::Literal(arg) => args.push(arg),
                    _ => unreachable!(),
                };
            }

            take_token(tokens, &TokenContents::CloseParen)?;

            Ok(Statement::Function(Function::new(
                name,
                args,
                parse_block(tokens)?,
            )))
        }
        _ => Err(unexpected_token(tokens)),
    }
}

/// Parses an if-statement.
fn parse_if_statement(tokens: &mut TokenCursor) -> Result<Statement, ParseError> {
    take_literal(tokens, "if")?;

    // Parse the initial condition and branch.
    let mut conditions = vec![parse_and_or(tokens)?];
    let mut branches = vec![parse_block(tokens)?];

    loop {
        if take_literal(tokens, "else").is_err() {
            break;
        }

        if take_literal(tokens, "if").is_ok() {
            conditions.push(parse_and_or(tokens)?);
            branches.push(parse_block(tokens)?);
            continue;
        }

        branches.push(parse_block(tokens)?);
        break;
    }

    Ok(Statement::If(ConditionalChain {
        conditions,
        branches,
    }))
}

/// Parses a for-loop.
pub(crate) fn parse_for_loop(tokens: &mut TokenCursor) -> Result<Statement, ParseError> {
    take_literal(tokens, "for")?;

    let variable = match parse_word(tokens) {
        Ok(Word::Literal(literal)) => literal,
        Ok(_) => return Err(ParseError::InvalidSyntax("expected literal".to_owned())),
        Err(error) => return Err(error),
    };

    take_literal(tokens, "in")?;
    let in_word = tokens.next_if(|t| matches!(t.contents, TokenContents::Literal(_)));

    // Determine an abstract iteration rule if the loop is a for-in-of-loop.
    if in_word.is_some() && take_literal(tokens, "of").is_ok() {
        let iterable = parse_word(tokens)?;
        let body = parse_block(tokens)?;
        return Ok(Statement::ForOfIn(ForOfIterableLoop {
            variable,
            iteration_rule: iteration_rule(&in_word.expect("has iteration rule"))?,
            iterable,
            body,
        }));
    }

    // Extract the concrete iterable if the loop is a normal for-in-loop.
    let iterable = if let Some(TokenContents::Literal(literal)) = in_word.map(|t| t.contents) {
        parse_iterable(&literal)?
    } else if let Ok(list) = parse_list(tokens) {
        Iterable::from(list)
    } else {
        match parse_word(tokens) {
            Ok(Word::Literal(literal)) => parse_iterable(&literal)?,
            Ok(_) => return Err(ParseError::InvalidSyntax("expected iterable".to_owned())),
            Err(error) => return Err(error),
        }
    };

    let body = parse_block(tokens)?;

    Ok(Statement::ForIn(ForIterableLoop {
        variable,
        iterable,
        body,
    }))
}

/// Parses a while-loop.
fn parse_while_loop(tokens: &mut TokenCursor) -> ParseResult<Statement> {
    take_literal(tokens, "while")?;

    Ok(Statement::While(ConditionalLoop {
        condition: parse_and_or(tokens)?,
        body: parse_block(tokens)?,
    }))
}

/// Parses a code block surrounded by curly braces.
fn parse_block(tokens: &mut TokenCursor) -> ParseResult<Block> {
    take_token(tokens, &TokenContents::OpenBrace)?;

    let mut block = Block::default();
    loop {
        match &tokens.peek().contents {
            TokenContents::Eol => skip_newlines(tokens),
            TokenContents::Eof => return Err(ParseError::IncompleteSequence),
            TokenContents::CloseBrace => break,
            _ => {
                block.statement(parse_statement(tokens)?);
            }
        }
    }

    take_token(tokens, &TokenContents::CloseBrace)?;

    Ok(block)
}

#[cfg(test)]
mod tests {
    use pjsh_ast::{AndOr, Command, IterationRule, List, Pipeline, PipelineSegment};

    use crate::{token::Token, Span};

    use super::*;

    #[test]
    fn it_parses_assignment_statements() {
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("key".into()), Span::new(0, 3)),
                Token::new(TokenContents::Assign, Span::new(4, 5)),
                Token::new(TokenContents::Literal("value".into()), Span::new(6, 11)),
            ])),
            Ok(Statement::Assignment(Assignment {
                key: Word::Literal("key".into()),
                value: Word::Literal("value".into()),
            }))
        )
    }

    #[test]
    fn parse_function_statement() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("fn".into()), span),
                Token::new(TokenContents::Literal("function_name".into()), span),
                Token::new(TokenContents::OpenParen, span),
                Token::new(TokenContents::Literal("arg".into()), span),
                Token::new(TokenContents::CloseParen, span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Literal("test".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Statement::Function(Function {
                name: "function_name".into(),
                args: vec!["arg".into()],
                body: Block {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![
                                    Word::Literal("echo".into()),
                                    Word::Literal("test".into())
                                ],
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
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("if".into()), span),
                Token::new(TokenContents::Literal("true".into()), span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Literal("test".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Statement::If(ConditionalChain {
                conditions: vec![AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            arguments: vec![Word::Literal("true".into())],
                            redirects: Vec::new(),
                        })]
                    }]
                }],
                branches: vec![Block {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![
                                    Word::Literal("echo".into()),
                                    Word::Literal("test".into())
                                ],
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
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("if".into()), span),
                Token::new(TokenContents::Literal("false".into()), span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Literal("first".into()), span),
                Token::new(TokenContents::CloseBrace, span),
                Token::new(TokenContents::Literal("else".into()), span),
                Token::new(TokenContents::Literal("if".into()), span),
                Token::new(TokenContents::Literal("false".into()), span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Literal("second".into()), span),
                Token::new(TokenContents::CloseBrace, span),
                Token::new(TokenContents::Literal("else".into()), span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Literal("third".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Statement::If(ConditionalChain {
                conditions: vec![
                    AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![Word::Literal("false".into())],
                                redirects: Vec::new(),
                            })]
                        }]
                    },
                    AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![Word::Literal("false".into())],
                                redirects: Vec::new(),
                            })]
                        }]
                    }
                ],
                branches: vec![
                    Block {
                        statements: vec![Statement::AndOr(AndOr {
                            operators: Vec::new(),
                            pipelines: vec![Pipeline {
                                is_async: false,
                                segments: vec![PipelineSegment::Command(Command {
                                    arguments: vec![
                                        Word::Literal("echo".into()),
                                        Word::Literal("first".into())
                                    ],
                                    redirects: Vec::new(),
                                })]
                            }]
                        })]
                    },
                    Block {
                        statements: vec![Statement::AndOr(AndOr {
                            operators: Vec::new(),
                            pipelines: vec![Pipeline {
                                is_async: false,
                                segments: vec![PipelineSegment::Command(Command {
                                    arguments: vec![
                                        Word::Literal("echo".into()),
                                        Word::Literal("second".into())
                                    ],
                                    redirects: Vec::new(),
                                })]
                            }]
                        })]
                    },
                    Block {
                        statements: vec![Statement::AndOr(AndOr {
                            operators: Vec::new(),
                            pipelines: vec![Pipeline {
                                is_async: false,
                                segments: vec![PipelineSegment::Command(Command {
                                    arguments: vec![
                                        Word::Literal("echo".into()),
                                        Word::Literal("third".into())
                                    ],
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
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("while".into()), span),
                Token::new(TokenContents::Literal("false".into()), span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Literal("test".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Statement::While(ConditionalLoop {
                condition: AndOr {
                    operators: Vec::new(),
                    pipelines: vec![Pipeline {
                        is_async: false,
                        segments: vec![PipelineSegment::Command(Command {
                            arguments: vec![Word::Literal("false".into())],
                            redirects: Vec::new(),
                        })]
                    }]
                },
                body: Block {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![
                                    Word::Literal("echo".into()),
                                    Word::Literal("test".into())
                                ],
                                redirects: Vec::new(),
                            })]
                        }]
                    })]
                }
            }))
        )
    }

    #[test]
    fn parse_for_in_loop() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_for_loop(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("for".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("i".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("in".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::OpenBracket, span),
                Token::new(TokenContents::Literal("a".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("b".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("c".into()), span),
                Token::new(TokenContents::CloseBracket, span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Variable("i".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Statement::ForIn(ForIterableLoop {
                variable: "i".into(),
                iterable: pjsh_ast::Iterable::from(List::from(vec![
                    Word::Literal("a".into()),
                    Word::Literal("b".into()),
                    Word::Literal("c".into()),
                ])),
                body: Block {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![
                                    Word::Literal("echo".into()),
                                    Word::Variable("i".into())
                                ],
                                redirects: Vec::new(),
                            })]
                        }]
                    })]
                }
            }))
        );
    }

    #[test]
    fn parse_for_of_in_loop() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_for_loop(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("for".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("color".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("in".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("words".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("of".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("red green blue".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Variable("color".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Statement::ForOfIn(ForOfIterableLoop {
                variable: "color".into(),
                iteration_rule: IterationRule::Words,
                iterable: Word::Literal("red green blue".into()),
                body: Block {
                    statements: vec![Statement::AndOr(AndOr {
                        operators: Vec::new(),
                        pipelines: vec![Pipeline {
                            is_async: false,
                            segments: vec![PipelineSegment::Command(Command {
                                arguments: vec![
                                    Word::Literal("echo".into()),
                                    Word::Variable("color".into())
                                ],
                                redirects: Vec::new(),
                            })]
                        }]
                    })]
                }
            }))
        );
    }

    #[test]
    fn parse_for_invalid_of_in_loop() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert!(parse_for_loop(&mut TokenCursor::from(vec![
            Token::new(TokenContents::Literal("for".into()), span),
            Token::new(TokenContents::Whitespace, span),
            Token::new(TokenContents::Literal("color".into()), span),
            Token::new(TokenContents::Whitespace, span),
            Token::new(TokenContents::Literal("in".into()), span),
            Token::new(TokenContents::Whitespace, span),
            Token::new(TokenContents::Literal("INVALID".into()), span),
            Token::new(TokenContents::Whitespace, span),
            Token::new(TokenContents::Literal("of".into()), span),
            Token::new(TokenContents::Whitespace, span),
            Token::new(TokenContents::Literal("red green blue".into()), span),
            Token::new(TokenContents::Whitespace, span),
            Token::new(TokenContents::OpenBrace, span),
            Token::new(TokenContents::Literal("echo".into()), span),
            Token::new(TokenContents::Whitespace, span),
            Token::new(TokenContents::Variable("color".into()), span),
            Token::new(TokenContents::CloseBrace, span),
        ]))
        .is_err());
    }

    #[test]
    fn parse_statement_before_unexpected() {
        let span = Span::new(0, 0); // Does not matter during this test.

        let mut tokens = TokenCursor::from(vec![
            Token::new(TokenContents::Literal("echo".into()), span),
            Token::new(TokenContents::Literal("test".into()), span),
            Token::new(TokenContents::CloseParen, span), // Unexpected token.
        ]);

        // First, return the valid statement.
        assert_eq!(
            parse_statement(&mut tokens),
            Ok(Statement::AndOr(AndOr {
                operators: Vec::new(),
                pipelines: vec![Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment::Command(Command {
                        arguments: vec![Word::Literal("echo".into()), Word::Literal("test".into())],
                        redirects: Vec::new(),
                    })]
                }]
            }))
        );

        // Then, return the parse error.
        assert_eq!(
            parse_statement(&mut tokens),
            Err(ParseError::UnexpectedToken(Token {
                contents: TokenContents::CloseParen,
                span
            }))
        );
    }
}
