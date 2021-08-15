use crate::{
    ast::{AndOr, AndOrPart, List, ListPart, PipeSequence, Pipeline},
    token::{Token, Unit},
};

use super::{
    adapter::LexerAdapter,
    command::CommandParser,
    error::ParseError,
    meta::{LinebreakParser, SeparatorOpParser},
    Parse,
};

/// Parses [`crate::ast::AndOr`] syntax.
///```yacc
/// and_or           :                         pipeline
///                  | and_or AND_IF linebreak pipeline
///                  | and_or OR_IF  linebreak pipeline
///                  ;
///```
pub(crate) struct AndOrParser {
    pipeline_parser: PipelineParser,
    linebreak_parser: LinebreakParser,
}

impl AndOrParser {
    pub fn new(pipeline_parser: PipelineParser, linebreak_parser: LinebreakParser) -> Self {
        Self {
            pipeline_parser,
            linebreak_parser,
        }
    }
}

impl Parse for AndOrParser {
    type Item = AndOr;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut parts = vec![AndOrPart::Start(self.pipeline_parser.parse(lexer)?)];

        loop {
            match lexer.peek_token() {
                Token::AndIf => {
                    lexer.next_token();
                    self.linebreak_parser.parse(lexer)?;
                    parts.push(AndOrPart::And(self.pipeline_parser.parse(lexer)?));
                }
                Token::OrIf => {
                    lexer.next_token();
                    self.linebreak_parser.parse(lexer)?;
                    parts.push(AndOrPart::Or(self.pipeline_parser.parse(lexer)?));
                }
                _ => break,
            }
        }

        if parts.is_empty() {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        } else {
            Ok(AndOr(parts))
        }
    }
}

/// Parses [`crate::ast::List`] syntax.
///```yacc
/// list             : list separator_op and_or
///                  |                   and_or
///                  ;
///```
pub(crate) struct ListParser {
    and_or_parser: AndOrParser,
    separator_op_parser: SeparatorOpParser,
}

impl ListParser {
    pub fn new(and_or_parser: AndOrParser, separator_op_parser: SeparatorOpParser) -> Self {
        Self {
            and_or_parser,
            separator_op_parser,
        }
    }
}

impl Parse for ListParser {
    type Item = List;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut parts = vec![ListPart::Start(self.and_or_parser.parse(lexer)?)];

        while let Ok(separator_op) = self.separator_op_parser.parse(lexer) {
            parts.push(ListPart::Tail(
                self.and_or_parser.parse(lexer)?,
                separator_op,
            ));
        }

        if parts.is_empty() {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        } else {
            Ok(List(parts))
        }
    }
}

/// Parses [`crate::ast::Pipeline`] syntax.
///```yacc
/// pipeline         :      pipe_sequence
///                  | Bang pipe_sequence
///                  ;
///```
pub(crate) struct PipelineParser {
    pipe_sequence_parser: PipeSequenceParser,
}

impl PipelineParser {
    pub fn new(pipe_sequence_parser: PipeSequenceParser) -> Self {
        Self {
            pipe_sequence_parser,
        }
    }

    fn is_bang(units: &[Unit]) -> bool {
        if units.len() != 1 {
            return false;
        }

        if let Some(Unit::Literal(literal)) = units.first() {
            return literal == "!";
        }

        false
    }
}

impl Parse for PipelineParser {
    type Item = Pipeline;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        match lexer.peek_token() {
            Token::Word(units) if Self::is_bang(units) => {
                lexer.next_token();
                Ok(Pipeline::Bang(self.pipe_sequence_parser.parse(lexer)?))
            }
            _ => Ok(Pipeline::Normal(self.pipe_sequence_parser.parse(lexer)?)),
        }
    }
}

/// Parses [`crate::ast::PipeSequence`] syntax.
///```yacc
/// pipe_sequence    :                             command
///                  | pipe_sequence '|' linebreak command
///                  ;
///```
pub(crate) struct PipeSequenceParser {
    command_parser: CommandParser,
    linebreak_parser: LinebreakParser,
}

impl PipeSequenceParser {
    pub fn new(command_parser: CommandParser, linebreak_parser: LinebreakParser) -> Self {
        Self {
            command_parser,
            linebreak_parser,
        }
    }
}

impl Parse for PipeSequenceParser {
    type Item = PipeSequence;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut commands = vec![self.command_parser.parse(lexer)?];

        while lexer.peek_token() == &Token::Pipe {
            lexer.next_token();
            if self.linebreak_parser.parse(lexer).is_ok() {
                commands.push(self.command_parser.parse(lexer)?);
            }
        }

        if commands.is_empty() {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        } else {
            Ok(PipeSequence(commands))
        }
    }
}
