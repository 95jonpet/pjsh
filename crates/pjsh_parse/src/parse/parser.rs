use std::{fmt::Display, iter::Peekable, vec::IntoIter};

use crate::lex::lexer::{LexError, Span, Token};
use crate::tokens::{self, TokenContents};
use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Command, InterpolationUnit, Pipeline, PipelineSegment, Program,
    Statement, Word,
};

pub fn parse(src: &str) -> Result<Program<'_>, ParseError<'_>> {
    match crate::lex(src) {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            parser.parse_program()
        }
        Err(LexError::UnexpectedEof) => Err(ParseError::UnexpectedEof),
        Err(error) => {
            eprintln!("pjsh: {}", error);
            Err(ParseError::UnexpectedEof)
        }
    }
}

pub fn parse_interpolation(src: &str) -> Result<Word<'_>, ParseError<'_>> {
    match crate::lex_interpolation(src) {
        Ok(token) => {
            let mut parser = Parser::new(vec![token]);
            parser.parse_word()
        }
        Err(LexError::UnexpectedEof) => Err(ParseError::UnexpectedEof),
        Err(error) => {
            eprintln!("pjsh: {}", error);
            Err(ParseError::UnexpectedEof)
        }
    }
}

pub struct Parser<'a> {
    tokens: TokenCursor<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens: TokenCursor::new(tokens),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program<'a>, ParseError<'a>> {
        let mut statements = Vec::new();
        loop {
            match self.parse_statement() {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(ParseError::IncompleteSequence) => {
                    return Err(ParseError::IncompleteSequence);
                }
                _ => break,
            }
        }

        let program = Program { statements };

        if self.tokens.peek_token().contents != TokenContents::Eof {
            return Err(ParseError::UnexpectedToken(
                self.tokens.peek_token().clone(),
            ));
        }

        Ok(program)
    }

    pub fn parse_and_or(&mut self) -> Result<AndOr<'a>, ParseError<'a>> {
        let mut pipelines = vec![self.parse_pipeline()?];
        let mut operators = Vec::new();

        loop {
            let operator = match self.tokens.peek_token().contents {
                TokenContents::AndIf => AndOrOp::And,
                TokenContents::OrIf => AndOrOp::Or,
                _ => break,
            };
            self.tokens.next_token();

            operators.push(operator);
            pipelines.push(self.parse_pipeline()?);
        }

        Ok(AndOr {
            pipelines,
            operators,
        })
    }

    pub fn parse_pipeline(&mut self) -> Result<Pipeline<'a>, ParseError<'a>> {
        match self.tokens.peek_token().contents {
            TokenContents::PipeStart => {
                self.tokens.next_token();
                self.parse_smart_pipeline()
            }
            _ => self.parse_legacy_pipeline(),
        }
    }

    pub fn parse_legacy_pipeline(&mut self) -> Result<Pipeline<'a>, ParseError<'a>> {
        let mut segments = Vec::new();

        while let Ok(segment) = self.parse_pipeline_segment() {
            segments.push(segment);

            if self.tokens.peek_token().contents == TokenContents::Pipe {
                self.tokens.next_token();

                if self.tokens.peek_token().contents == TokenContents::Eol {
                    self.tokens.next_token();
                }
            } else {
                break;
            }
        }

        if segments.is_empty() {
            return Err(ParseError::UnexpectedToken(
                self.tokens.peek_token().clone(),
            ));
        }

        let mut is_async = false;
        if self.tokens.peek_token().contents == TokenContents::Amp {
            is_async = true;
            self.tokens.next_token();
        }

        Ok(Pipeline { is_async, segments })
    }

    pub fn parse_smart_pipeline(&mut self) -> Result<Pipeline<'a>, ParseError<'a>> {
        let mut segments = Vec::new();
        let mut is_async = false;

        loop {
            match self.tokens.peek_token().contents {
                TokenContents::Amp => {
                    self.tokens.next_token();
                    is_async = true;
                    break;
                }
                TokenContents::Semi => {
                    self.tokens.next_token();
                    break;
                }
                _ => segments.push(self.parse_pipeline_segment()?),
            }

            if self.tokens.peek_token().contents == TokenContents::Eol {
                self.tokens.next_token();
            }

            match self.tokens.peek_token().contents {
                TokenContents::Pipe => {
                    self.tokens.next_token();

                    if self.tokens.peek_token().contents == TokenContents::Eol {
                        self.tokens.next_token();
                    }
                }
                TokenContents::Eof => return Err(ParseError::IncompleteSequence),
                TokenContents::Amp => {
                    self.tokens.next_token();
                    is_async = true;
                    break;
                }
                TokenContents::Semi => {
                    self.tokens.next_token();
                    break;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.tokens.peek_token().clone(),
                    ))
                }
            }
        }

        if segments.is_empty() {
            return Err(ParseError::UnexpectedToken(
                self.tokens.peek_token().clone(),
            ));
        }

        Ok(Pipeline { is_async, segments })
    }

    pub fn parse_pipeline_segment(&mut self) -> Result<PipelineSegment<'a>, ParseError<'a>> {
        let command = self.parse_command()?;
        Ok(PipelineSegment { command })
    }

    pub fn parse_command(&mut self) -> Result<Command<'a>, ParseError<'a>> {
        let program = self.parse_word()?;
        let mut arguments = Vec::new();

        while let Ok(argument) = self.parse_word() {
            arguments.push(argument)
        }

        Ok(Command {
            program,
            arguments,
            redirects: Vec::new(),
        })
    }

    /// Tries to parse a [`Statement`] from the next tokens of input.
    pub fn parse_statement(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        self.skip_newlines();

        let mut assignment_iter = self.tokens.clone();
        assignment_iter.next_token();
        if assignment_iter.peek_token().contents == TokenContents::Assign {
            let key = self.parse_word()?;

            debug_assert_eq!(self.tokens.peek_token().contents, TokenContents::Assign);
            self.tokens.next_token();

            let value = self.parse_word()?;
            return Ok(Statement::Assignment(Assignment { key, value }));
        }

        Ok(Statement::AndOr(self.parse_and_or()?))
    }

    pub(crate) fn parse_word(&mut self) -> Result<Word<'a>, ParseError<'a>> {
        match self.tokens.peek_token().contents {
            TokenContents::Literal(literal) => {
                self.tokens.next_token();
                Ok(Word::Literal(literal))
            }
            TokenContents::TripleQuote => self.parse_triple_quoted(),
            TokenContents::Quote => self.parse_quoted(),
            TokenContents::Interpolation(_) => self.parse_interpolation(),
            TokenContents::Variable(variable) => {
                self.tokens.next_token();
                Ok(Word::Variable(variable))
            }

            TokenContents::Eof => Err(ParseError::UnexpectedEof),
            _ => Err(ParseError::UnexpectedToken(
                self.tokens.peek_token().clone(),
            )),
        }
    }

    fn parse_interpolation(&mut self) -> Result<Word<'a>, ParseError<'a>> {
        if let TokenContents::Interpolation(units) = self.tokens.next_token().contents {
            let word_units = units
                .into_iter()
                .map(|unit| self.parse_interpolation_unit(unit))
                .collect();
            Ok(Word::Interpolation(word_units))
        } else {
            Err(ParseError::UnexpectedToken(
                self.tokens.peek_token().clone(),
            ))
        }
    }

    fn parse_interpolation_unit(
        &self,
        unit: tokens::InterpolationUnit<'a>,
    ) -> InterpolationUnit<'a> {
        match unit {
            tokens::InterpolationUnit::Literal(literal) => InterpolationUnit::Literal(literal),
            tokens::InterpolationUnit::Unicode(ch) => InterpolationUnit::Unicode(ch),
            tokens::InterpolationUnit::Variable(var) => InterpolationUnit::Variable(var),
        }
    }

    fn parse_triple_quoted(&mut self) -> Result<Word<'a>, ParseError<'a>> {
        self.tokens.next_token();
        let mut quoted = String::new();
        loop {
            let token = self.tokens.next_token();
            match token.contents {
                TokenContents::TripleQuote => break,
                TokenContents::Quoted(string) => quoted.push_str(string),
                TokenContents::Eof => return Err(ParseError::UnexpectedEof),
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }

        let mut lines = quoted.trim_end().lines();
        let mut string = String::new();
        let mut indent: usize = 0;

        match lines.next() {
            Some(first_line) if first_line.is_empty() => (),
            _ => todo!("must contain at least one line."),
        }

        if let Some(line) = lines.next() {
            let trimmed = line.trim_start_matches(char::is_whitespace);
            indent = line.len() - trimmed.len();
            string.push_str(trimmed);
        }

        for line in lines {
            let prefix = &line[..indent];
            if prefix.contains(|ch: char| !ch.is_whitespace()) {
                todo!("unexpected indentation level");
            }

            string.push('\n');
            string.push_str(&line[indent..]);
        }

        Ok(Word::Quoted(string))
    }

    fn parse_quoted(&mut self) -> Result<Word<'a>, ParseError<'a>> {
        self.tokens.next_token();
        let mut quoted = String::new();
        loop {
            let token = self.tokens.next_token();
            match token.contents {
                TokenContents::Quote => break,
                TokenContents::Quoted(string) => quoted.push_str(string),
                TokenContents::Eof => return Err(ParseError::UnexpectedEof),
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }
        Ok(Word::Quoted(quoted))
    }

    fn skip_newlines(&mut self) {
        while self.tokens.peek_token().contents == TokenContents::Eol {
            self.tokens.next_token();
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    IncompleteSequence,
    UnexpectedEof,
    UnexpectedToken(Token<'a>),
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IncompleteSequence => write!(f, "incomplete sequence"),
            ParseError::UnexpectedEof => write!(f, "unexpected end of file"),
            ParseError::UnexpectedToken(token) => {
                write!(f, "unexpected token {:? }", token.contents)
            }
        }
    }
}

#[derive(Clone)]
struct TokenCursor<'a> {
    eof_token: Token<'a>,
    tokens: Peekable<IntoIter<Token<'a>>>,
}

impl<'a> TokenCursor<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            eof_token: Token::new(TokenContents::Eof, Span::new(0, 0)),
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn peek_token(&mut self) -> &Token<'a> {
        loop {
            let token = self.tokens.peek().unwrap_or(&self.eof_token);
            if matches!(
                token.contents,
                TokenContents::Comment | TokenContents::Whitespace
            ) {
                self.tokens.next();
            } else {
                break;
            }
        }

        self.tokens.peek().unwrap_or(&self.eof_token)
    }

    pub fn next_token(&mut self) -> Token<'a> {
        loop {
            let token = self.tokens.next().unwrap_or_else(|| self.eof_token.clone());
            self.eof_token = Token::new(
                TokenContents::Eof,
                Span::new(token.span.end, token.span.end),
            );
            match token.contents {
                TokenContents::Comment | TokenContents::Whitespace => {
                    continue;
                }
                _ => return token,
            }
        }
    }
}
