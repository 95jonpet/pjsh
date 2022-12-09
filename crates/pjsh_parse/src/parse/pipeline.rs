use pjsh_ast::{Pipeline, PipelineSegment};

use crate::{token::TokenContents, ParseError};

use super::{
    command::parse_command, condition::parse_condition, cursor::TokenCursor,
    utils::unexpected_token, ParseResult,
};

/// Parses a pipeline. Handles both smart pipelines and legacy pipelines.
pub fn parse_pipeline(tokens: &mut TokenCursor) -> ParseResult<Pipeline> {
    if tokens.next_if_eq(TokenContents::PipeStart).is_some() {
        return parse_smart_pipeline(tokens);
    }

    parse_legacy_pipeline(tokens)
}

/// Parses a legacy [`Pipeline`] without an explicit start and end.
pub fn parse_legacy_pipeline(tokens: &mut TokenCursor) -> ParseResult<Pipeline> {
    let mut pipeline = Pipeline::default();

    // No input to parse - a valid legacy pipeline cannot be constructed.
    if tokens.peek().contents == TokenContents::Eof {
        return Err(ParseError::UnexpectedEof);
    }

    loop {
        match parse_pipeline_segment(tokens) {
            // Continually add segments until there is no more input.
            Ok(segment) => {
                pipeline.segments.push(segment);

                if tokens.next_if_eq(TokenContents::Pipe).is_none() {
                    // Legacy pipelines end when there are no more pipes.
                    break;
                } else {
                    tokens.next_if_eq(TokenContents::Eol);
                }
            }

            // A legacy pipeline is automatically terminated at the end of input.
            Err(ParseError::UnexpectedEof) => break,

            // Other parser errors must be returned and handled elsewhere.
            Err(error) => return Err(error),
        }
    }

    pipeline.is_async = tokens.next_if_eq(TokenContents::Amp).is_some();

    Ok(pipeline)
}

/// Parses a "smart" [`Pipeline`] with an explicit start and end.
pub fn parse_smart_pipeline(tokens: &mut TokenCursor) -> ParseResult<Pipeline> {
    tokens.newline_is_whitespace(true); // Newline is trivialized in a smart pipeline.
    let mut pipeline = Pipeline::default();

    loop {
        match tokens.peek().contents {
            TokenContents::Amp => {
                tokens.next();
                pipeline.is_async = true;
                break;
            }
            TokenContents::Semi => {
                tokens.next();
                break;
            }
            _ => pipeline.segments.push(parse_pipeline_segment(tokens)?),
        }

        match tokens.peek().contents {
            TokenContents::Pipe => {
                tokens.next();
            }
            TokenContents::Eof => return Err(ParseError::IncompleteSequence),
            TokenContents::Amp => {
                tokens.next();
                pipeline.is_async = true;
                break;
            }
            TokenContents::Semi => {
                tokens.next();
                break;
            }
            _ => {
                tokens.newline_is_whitespace(false); // Ensure a clean exit.
                return Err(unexpected_token(tokens));
            }
        }
    }

    tokens.newline_is_whitespace(false); // Ensure a clean exit.

    // A pipeline is only valid if it contains one or more segments.
    if pipeline.segments.is_empty() {
        return Err(unexpected_token(tokens));
    }

    Ok(pipeline)
}

/// Parses a pipeline segment.
pub fn parse_pipeline_segment(tokens: &mut TokenCursor) -> ParseResult<PipelineSegment> {
    if let Ok(condition) = parse_condition(tokens) {
        return Ok(PipelineSegment::Condition(condition));
    }

    Ok(PipelineSegment::Command(parse_command(tokens)?))
}

#[cfg(test)]
mod tests {
    use pjsh_ast::{Command, Word};

    use crate::{token::Token, Span};

    use super::*;

    #[test]
    fn parse_legacy_pipeline() {
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("first".into()), Span::new(0, 5)),
                Token::new(TokenContents::Literal("second".into()), Span::new(6, 12)),
                Token::new(TokenContents::Pipe, Span::new(13, 14)),
                Token::new(TokenContents::Literal("third".into()), Span::new(15, 20)),
            ])),
            Ok(Pipeline {
                is_async: false,
                segments: vec![
                    PipelineSegment::Command(Command {
                        arguments: vec![
                            Word::Literal("first".into()),
                            Word::Literal("second".into())
                        ],
                        redirects: Vec::new(),
                    }),
                    PipelineSegment::Command(Command {
                        arguments: vec![Word::Literal("third".into())],
                        redirects: Vec::new(),
                    }),
                ]
            })
        );
    }

    #[test]
    fn parse_legacy_pipeline_async() {
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::Literal("command".into()), Span::new(0, 7)),
                Token::new(TokenContents::Amp, Span::new(7, 8)),
            ])),
            Ok(Pipeline {
                is_async: true,
                segments: vec![PipelineSegment::Command(Command {
                    arguments: vec![Word::Literal("command".into())],
                    redirects: Vec::new(),
                })]
            })
        );
    }

    #[test]
    fn parse_smart_pipeline() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::PipeStart, span),
                Token::new(TokenContents::Literal("cmd1".into()), span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::Pipe, span),
                Token::new(TokenContents::Literal("cmd2".into()), span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::Semi, span),
            ])),
            Ok(Pipeline {
                is_async: false,
                segments: vec![
                    PipelineSegment::Command(Command {
                        arguments: vec![Word::Literal("cmd1".into())],
                        redirects: Vec::new(),
                    }),
                    PipelineSegment::Command(Command {
                        arguments: vec![Word::Literal("cmd2".into())],
                        redirects: Vec::new(),
                    }),
                ]
            })
        );
    }

    #[test]
    fn parse_smart_pipeline_async() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::PipeStart, span),
                Token::new(TokenContents::Literal("cmd1".into()), span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::Pipe, span),
                Token::new(TokenContents::Literal("cmd2".into()), span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::Amp, span),
            ])),
            Ok(Pipeline {
                is_async: true,
                segments: vec![
                    PipelineSegment::Command(Command {
                        arguments: vec![Word::Literal("cmd1".into())],
                        redirects: Vec::new(),
                    }),
                    PipelineSegment::Command(Command {
                        arguments: vec![Word::Literal("cmd2".into())],
                        redirects: Vec::new(),
                    }),
                ]
            })
        );
    }

    #[test]
    fn parse_smart_pipeline_whitespace() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::PipeStart, span),
                Token::new(TokenContents::Literal("cmd".into()), span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::Literal("arg1".into()), span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::Literal("arg2".into()), span),
                Token::new(TokenContents::Eol, span),
                Token::new(TokenContents::Semi, span),
            ])),
            Ok(Pipeline {
                is_async: false,
                segments: vec![PipelineSegment::Command(Command {
                    arguments: vec![
                        Word::Literal("cmd".into()),
                        Word::Literal("arg1".into()),
                        Word::Literal("arg2".into())
                    ],
                    redirects: Vec::new(),
                })]
            })
        );
    }

    #[test]
    fn parse_smart_pipeline_partial() {
        let span = Span::new(0, 0); // Does not matter during this test.

        let mut tokens = vec![
            Token::new(TokenContents::PipeStart, span),
            Token::new(TokenContents::Whitespace, span),
            Token::new(TokenContents::Literal("cmd1".into()), span),
            Token::new(TokenContents::Eol, span),
        ];
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(tokens.clone())),
            Err(ParseError::IncompleteSequence)
        );

        tokens.push(Token::new(TokenContents::Pipe, span));
        tokens.push(Token::new(TokenContents::Whitespace, span));
        tokens.push(Token::new(TokenContents::Literal("cmd2".into()), span));
        tokens.push(Token::new(TokenContents::Eol, span));
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(tokens.clone())),
            Err(ParseError::IncompleteSequence)
        );

        tokens.push(Token::new(TokenContents::Semi, span));
        assert!(parse_pipeline(&mut TokenCursor::from(tokens)).is_ok());
    }

    #[test]
    fn parse_smart_async_pipeline() {
        let span = Span::new(0, 0); // Does not matter during this test.
        assert_eq!(
            parse_pipeline(&mut TokenCursor::from(vec![
                Token::new(TokenContents::PipeStart, span),
                Token::new(TokenContents::Literal("command".into()), span),
                Token::new(TokenContents::Amp, span), // Should mark the end of the pipeline.
            ])),
            Ok(Pipeline {
                is_async: true,
                segments: vec![PipelineSegment::Command(Command {
                    arguments: vec![Word::Literal("command".into())],
                    redirects: Vec::new(),
                })]
            })
        );
    }
}
