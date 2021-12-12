use std::fmt::Display;

use crate::lex::lexer::{LexError, Token};
use crate::tokens::{self, TokenContents};
use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Command, FileDescriptor, InterpolationUnit, Pipeline,
    PipelineSegment, Program, Redirect, RedirectOperator, Statement, Word,
};

use super::cursor::TokenCursor;

/// Tries to parse a [`Program`] by consuming some input `src` in its entirety.
/// A [`ParserError`] is returned if a program can't be parsed.
pub fn parse(src: &str) -> Result<Program, ParseError> {
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

/// Tries to parse a [`Word`] from within an interpolation.
/// A [`ParserError`] is returned if a program can't be parsed.
pub fn parse_interpolation(src: &str) -> Result<Word, ParseError> {
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

/// A parser creates an abstract syntax tree from a tokenized input.
pub struct Parser {
    tokens: TokenCursor,
}

impl Parser {
    /// Constructs a new parser for parsing some tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: TokenCursor::new(tokens),
        }
    }

    /// Parses [`Program`] by consuming all remaining input.
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::new();
        while let Ok(statement) = self.parse_statement() {
            program.statement(statement);
        }

        // All tokens should be consumed when parsing a valid program.
        if self.tokens.peek().contents != TokenContents::Eof {
            return Err(self.unexpected_token());
        }

        Ok(program)
    }

    pub fn parse_and_or(&mut self) -> Result<AndOr, ParseError> {
        let mut pipelines = vec![self.parse_pipeline()?];
        let mut operators = Vec::new();

        loop {
            // Semi tokens terminate the current statement.
            if self.tokens.peek().contents == TokenContents::Semi {
                self.tokens.next();
                break;
            }

            let operator = match self.tokens.peek().contents {
                TokenContents::AndIf => AndOrOp::And,
                TokenContents::OrIf => AndOrOp::Or,
                _ => break,
            };
            self.tokens.next();

            operators.push(operator);
            pipelines.push(self.parse_pipeline()?);
        }

        Ok(AndOr {
            pipelines,
            operators,
        })
    }

    pub fn parse_pipeline(&mut self) -> Result<Pipeline, ParseError> {
        match self.tokens.peek().contents {
            TokenContents::PipeStart => {
                self.tokens.next();
                self.parse_smart_pipeline()
            }
            _ => self.parse_legacy_pipeline(),
        }
    }

    pub fn parse_legacy_pipeline(&mut self) -> Result<Pipeline, ParseError> {
        let mut segments = Vec::new();

        while let Ok(segment) = self.parse_pipeline_segment() {
            segments.push(segment);

            if self.tokens.peek().contents == TokenContents::Pipe {
                self.tokens.next();

                if self.tokens.peek().contents == TokenContents::Eol {
                    self.tokens.next();
                }
            } else {
                break;
            }
        }

        if segments.is_empty() {
            return Err(self.unexpected_token());
        }

        let mut is_async = false;
        if self.tokens.peek().contents == TokenContents::Amp {
            is_async = true;
            self.tokens.next();
        }

        Ok(Pipeline { is_async, segments })
    }

    pub fn parse_smart_pipeline(&mut self) -> Result<Pipeline, ParseError> {
        let mut segments = Vec::new();
        let mut is_async = false;

        loop {
            match self.tokens.peek().contents {
                TokenContents::Amp => {
                    self.tokens.next();
                    is_async = true;
                    break;
                }
                TokenContents::Semi => {
                    self.tokens.next();
                    break;
                }
                _ => segments.push(self.parse_pipeline_segment()?),
            }

            if self.tokens.peek().contents == TokenContents::Eol {
                self.tokens.next();
            }

            match self.tokens.peek().contents {
                TokenContents::Pipe => {
                    self.tokens.next();

                    if self.tokens.peek().contents == TokenContents::Eol {
                        self.tokens.next();
                    }
                }
                TokenContents::Eof => return Err(ParseError::IncompleteSequence),
                TokenContents::Amp => {
                    self.tokens.next();
                    is_async = true;
                    break;
                }
                TokenContents::Semi => {
                    self.tokens.next();
                    break;
                }
                _ => return Err(self.unexpected_token()),
            }
        }

        if segments.is_empty() {
            return Err(self.unexpected_token());
        }

        Ok(Pipeline { is_async, segments })
    }

    pub fn parse_pipeline_segment(&mut self) -> Result<PipelineSegment, ParseError> {
        let command = self.parse_command()?;
        Ok(PipelineSegment { command })
    }

    /// Tries to parse a [`Command`] from the next tokens of input.
    pub fn parse_command(&mut self) -> Result<Command, ParseError> {
        let prefix_redirects = self.parse_redirects();
        let mut command = Command::new(self.parse_word()?);

        while let Ok(argument) = self.parse_word() {
            command.arg(argument);
        }

        for redirect in prefix_redirects {
            command.redirect(redirect);
        }
        for redirect in self.parse_redirects() {
            command.redirect(redirect);
        }

        Ok(command)
    }

    /// Tries to parse a [`Statement`] from the next tokens of input.
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        self.skip_newlines();

        let mut assignment_iter = self.tokens.clone();
        assignment_iter.next();
        if assignment_iter.peek().contents == TokenContents::Assign {
            let key = self.parse_word()?;

            debug_assert_eq!(self.tokens.peek().contents, TokenContents::Assign);
            self.tokens.next();

            let value = self.parse_word()?;
            return Ok(Statement::Assignment(Assignment { key, value }));
        }

        Ok(Statement::AndOr(self.parse_and_or()?))
    }

    pub(crate) fn parse_word(&mut self) -> Result<Word, ParseError> {
        match &self.tokens.peek().contents {
            TokenContents::Literal(_) => {
                let next = self.tokens.next();
                if let TokenContents::Literal(literal) = next.contents {
                    Ok(Word::Literal(literal))
                } else {
                    Err(ParseError::UnexpectedToken(next))
                }
            }
            TokenContents::TripleQuote => self.parse_triple_quoted(),
            TokenContents::Quote => self.parse_quoted(),
            TokenContents::Interpolation(_) => self.parse_interpolation(),
            TokenContents::Variable(_) => {
                let next = self.tokens.next();
                if let TokenContents::Variable(variable) = next.contents {
                    Ok(Word::Variable(variable))
                } else {
                    Err(ParseError::UnexpectedToken(next))
                }
            }

            TokenContents::Eof => Err(ParseError::UnexpectedEof),
            _ => Err(self.unexpected_token()),
        }
    }

    fn parse_redirects(&mut self) -> Vec<Redirect> {
        let mut redirects = Vec::new();
        while let Ok(redirect) = self.parse_redirect() {
            redirects.push(redirect);
        }
        redirects
    }

    pub(crate) fn parse_redirect(&mut self) -> Result<Redirect, ParseError> {
        match self.tokens.peek().contents {
            TokenContents::FdReadTo(fd) => {
                self.tokens.next();
                Ok(Redirect::new(
                    FileDescriptor::File(self.parse_word()?),
                    RedirectOperator::Write,
                    FileDescriptor::Number(fd),
                ))
            }
            TokenContents::FdWriteFrom(fd) => {
                self.tokens.next();
                Ok(Redirect::new(
                    FileDescriptor::Number(fd),
                    RedirectOperator::Write,
                    FileDescriptor::File(self.parse_word()?),
                ))
            }
            TokenContents::FdAppendFrom(fd) => {
                self.tokens.next();
                Ok(Redirect::new(
                    FileDescriptor::Number(fd),
                    RedirectOperator::Append,
                    FileDescriptor::File(self.parse_word()?),
                ))
            }
            _ => Err(self.unexpected_token()),
        }
    }

    fn parse_interpolation(&mut self) -> Result<Word, ParseError> {
        if let TokenContents::Interpolation(units) = self.tokens.next().contents {
            let word_units = units
                .into_iter()
                .map(|unit| self.parse_interpolation_unit(unit))
                .collect();
            Ok(Word::Interpolation(word_units))
        } else {
            Err(self.unexpected_token())
        }
    }

    fn parse_interpolation_unit(&self, unit: tokens::InterpolationUnit) -> InterpolationUnit {
        match unit {
            tokens::InterpolationUnit::Literal(literal) => InterpolationUnit::Literal(literal),
            tokens::InterpolationUnit::Unicode(ch) => InterpolationUnit::Unicode(ch),
            tokens::InterpolationUnit::Variable(var) => InterpolationUnit::Variable(var),
        }
    }

    fn parse_triple_quoted(&mut self) -> Result<Word, ParseError> {
        self.tokens.next();
        let mut quoted = String::new();
        loop {
            let token = self.tokens.next();
            match token.contents {
                TokenContents::TripleQuote => break,
                TokenContents::Quoted(string) => quoted.push_str(&string),
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

    fn parse_quoted(&mut self) -> Result<Word, ParseError> {
        self.tokens.next();
        let mut quoted = String::new();
        loop {
            let token = self.tokens.next();
            match token.contents {
                TokenContents::Quote => break,
                TokenContents::Quoted(string) => quoted.push_str(&string),
                TokenContents::Eof => return Err(ParseError::UnexpectedEof),
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }
        Ok(Word::Quoted(quoted))
    }

    /// Advances the token cursor until the next token is not an end-of-line token.
    fn skip_newlines(&mut self) {
        while self.tokens.peek().contents == TokenContents::Eol {
            self.tokens.next();
        }
    }

    /// Returns a [`ParseError::UnexpectedToken`] around a copy of the next token.
    fn unexpected_token(&mut self) -> ParseError {
        ParseError::UnexpectedToken(self.tokens.peek().clone())
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    IncompleteSequence,
    UnexpectedEof,
    UnexpectedToken(Token),
}

impl Display for ParseError {
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
