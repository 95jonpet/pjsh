use pjsh_ast::{
    Assignment, Block, ConditionalChain, ConditionalLoop, ForIterableLoop, ForOfIterableLoop,
    Function, Iterable, Statement, Switch, Value, Word,
};

use crate::{
    parse::{utils::sequence, word::parse_word},
    token::TokenContents,
    ParseError,
};

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

    // Try to parse a switch-statement.
    match parse_switch_statement(tokens) {
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
    match parse_assignment(tokens) {
        Ok(function_statement) => return Ok(function_statement),
        Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
        _ => (),
    }

    Ok(Statement::AndOr(parse_and_or(tokens)?))
}

/// Parses an assignment statement.
fn parse_assignment(tokens: &mut TokenCursor) -> ParseResult<Statement> {
    let mut peek = tokens.clone();
    let key = parse_word(&mut peek)?;
    take_token(&mut peek, &TokenContents::Assign)?;

    // Parse a single word value assignment.
    if let Ok(value) = parse_word(&mut peek) {
        *tokens = peek;
        return Ok(Statement::Assignment(Assignment {
            key,
            value: Value::Word(value),
        }));
    }

    // Parse a list value assignment.
    let list = parse_list(&mut peek)?;
    *tokens = peek;
    Ok(Statement::Assignment(Assignment {
        key,
        value: Value::List(list),
    }))
}

/// Parses a function declaration,
fn parse_function(tokens: &mut TokenCursor) -> ParseResult<Statement> {
    take_literal(tokens, "fn")?;

    match tokens.next().contents {
        TokenContents::Literal(name) => {
            take_token(tokens, &TokenContents::OpenParen)?;

            // Parse argument list.
            let mut args = Vec::new();
            let mut list_arg = None;
            while let Some(token) =
                tokens.next_if(|t| matches!(&t.contents, &TokenContents::Literal(_)))
            {
                match token.contents {
                    TokenContents::Literal(arg) if arg.ends_with("...") => {
                        list_arg = Some(arg.trim_end_matches("...").to_owned());
                        break; // Only a single list type argument is allowed.
                    }
                    TokenContents::Literal(arg) => args.push(arg),
                    _ => unreachable!(),
                };
            }

            take_token(tokens, &TokenContents::CloseParen)?;

            Ok(Statement::Function(Function::new(
                name,
                args,
                list_arg,
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

/// Parses a switch statement.
fn parse_switch_statement(tokens: &mut TokenCursor) -> ParseResult<Statement> {
    take_literal(tokens, "switch")?;
    sequence(tokens, |tokens| {
        let input = parse_word(tokens)?;

        take_token(tokens, &TokenContents::OpenBrace)?;
        skip_newlines(tokens);

        let mut branches = Vec::new();
        while take_token(tokens, &TokenContents::CloseBrace).is_err() {
            skip_newlines(tokens);
            let mut keys = Vec::new();

            // Parse one or more keys.
            keys.push(parse_word(tokens)?);
            while let Ok(word) = parse_word(tokens) {
                keys.push(word);
            }

            let body = parse_block(tokens)?;

            for key in keys {
                branches.push((key, body.clone()));
            }

            skip_newlines(tokens);
        }

        Ok(Statement::Switch(Switch { input, branches }))
    })
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
            Ok(Word::Variable(var)) => Iterable::Variable(var),
            Ok(_) => return Err(ParseError::InvalidSyntax("expected iterable".to_owned())),
            Err(ParseError::UnexpectedEof) => return Err(ParseError::IncompleteSequence),
            Err(error) => return Err(error),
        }
    };

    let body = parse_block(tokens).map_err(|err| match err {
        ParseError::UnexpectedEof => ParseError::IncompleteSequence,
        err => err,
    })?;

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
    use pjsh_ast::{AndOr, Command, IterationRule, List, Pipeline, PipelineSegment, Switch, Value};

    use crate::{token::Token, Span};

    use super::*;

    #[test]
    fn it_parses_word_assignments() {
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("key".into()), Span::new(0, 3)),
                Token::new(TokenContents::Assign, Span::new(4, 5)),
                Token::new(TokenContents::Literal("value".into()), Span::new(6, 11)),
            ])),
            Ok(Statement::Assignment(Assignment {
                key: Word::Literal("key".into()),
                value: Value::Word(Word::Literal("value".into())),
            }))
        )
    }

    #[test]
    fn it_parses_list_assignments() {
        let span = Span::new(0, 0);
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("key".into()), span),
                Token::new(TokenContents::Assign, span),
                Token::new(TokenContents::OpenBracket, span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::Literal("item1".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("item2".into()), span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::CloseBracket, span),
            ])),
            Ok(Statement::Assignment(Assignment {
                key: Word::Literal("key".into()),
                value: Value::List(List::from(vec![
                    Word::Literal("item1".into()),
                    Word::Literal("item2".into()),
                ])),
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
                list_arg: None,
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
    fn parse_incomplete_if_statement() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("if".into()), span),
                Token::new(TokenContents::Literal("true".into()), span),
                Token::new(TokenContents::OpenBrace, span), // Unexpected EOF after this.
            ])),
            Err(ParseError::IncompleteSequence)
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
    fn parse_switch_statement() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("switch".into()), span),
                Token::new(TokenContents::Literal("b".into()), span), // The input.
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("a".into()), span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("in_a".into()), span),
                Token::new(TokenContents::CloseBrace, span),
                Token::new(TokenContents::Literal("b".into()), span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("in_b".into()), span),
                Token::new(TokenContents::CloseBrace, span),
                Token::new(TokenContents::Literal("c".into()), span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("in_c".into()), span),
                Token::new(TokenContents::CloseBrace, span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Statement::Switch(Switch {
                input: Word::Literal("b".into()),
                branches: vec![
                    (
                        Word::Literal("a".into()),
                        Block {
                            statements: vec![Statement::AndOr(AndOr {
                                operators: Vec::new(),
                                pipelines: vec![Pipeline {
                                    is_async: false,
                                    segments: vec![PipelineSegment::Command(Command {
                                        arguments: vec![Word::Literal("in_a".into())],
                                        redirects: Vec::new(),
                                    })]
                                }]
                            })]
                        }
                    ),
                    (
                        Word::Literal("b".into()),
                        Block {
                            statements: vec![Statement::AndOr(AndOr {
                                operators: Vec::new(),
                                pipelines: vec![Pipeline {
                                    is_async: false,
                                    segments: vec![PipelineSegment::Command(Command {
                                        arguments: vec![Word::Literal("in_b".into())],
                                        redirects: Vec::new(),
                                    })]
                                }]
                            })]
                        }
                    ),
                    (
                        Word::Literal("c".into()),
                        Block {
                            statements: vec![Statement::AndOr(AndOr {
                                operators: Vec::new(),
                                pipelines: vec![Pipeline {
                                    is_async: false,
                                    segments: vec![PipelineSegment::Command(Command {
                                        arguments: vec![Word::Literal("in_c".into())],
                                        redirects: Vec::new(),
                                    })]
                                }]
                            })]
                        }
                    ),
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
    fn parse_incomplete_while_loop() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_statement(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("while".into()), span),
                Token::new(TokenContents::Literal("true".into()), span),
                Token::new(TokenContents::OpenBrace, span), // Unexpected EOF after this.
            ])),
            Err(ParseError::IncompleteSequence)
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
    fn parse_for_in_variable_loop() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_for_loop(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("for".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("item".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Literal("in".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Variable("items".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::OpenBrace, span),
                Token::new(TokenContents::Literal("echo".into()), span),
                Token::new(TokenContents::Whitespace, span),
                Token::new(TokenContents::Variable("i".into()), span),
                Token::new(TokenContents::CloseBrace, span),
            ])),
            Ok(Statement::ForIn(ForIterableLoop {
                variable: "item".into(),
                iterable: pjsh_ast::Iterable::Variable("items".into()),
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
    fn parse_incomplete_for_in_loop() {
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
                Token::new(TokenContents::Literal("c".into()), span), // Unexpected EOF after this.
            ])),
            Err(ParseError::IncompleteSequence)
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
