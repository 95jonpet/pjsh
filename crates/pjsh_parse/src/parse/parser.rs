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

        loop {
            match self.parse_statement() {
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
        if self.tokens.peek().contents != TokenContents::Eof {
            return Err(self.unexpected_token());
        }

        Ok(program)
    }

    /// Parses an [`AndOr`] consisting of one or more [`Pipeline`] definitions.
    pub fn parse_and_or(&mut self) -> Result<AndOr, ParseError> {
        let mut pipelines = vec![self.parse_pipeline()?];
        let mut operators = Vec::new();

        loop {
            if self.tokens.next_if_eq(TokenContents::Eof).is_some() {
                break;
            }

            // Semi tokens terminate the current statement.
            if self.tokens.next_if_eq(TokenContents::Semi).is_some() {
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

    /// Parses a pipeline. Handles both smart pipelines and legacy pipelines.
    pub fn parse_pipeline(&mut self) -> Result<Pipeline, ParseError> {
        if self.tokens.next_if_eq(TokenContents::PipeStart).is_some() {
            return self.parse_smart_pipeline();
        }

        self.parse_legacy_pipeline()
    }

    /// Parses a legacy [`Pipeline`] without an explicit start and end.
    pub fn parse_legacy_pipeline(&mut self) -> Result<Pipeline, ParseError> {
        let mut segments = Vec::new();

        // No input to parse - a valid legacy pipeline cannot be constructed.
        if self.tokens.peek().contents == TokenContents::Eof {
            return Err(ParseError::UnexpectedEof);
        }

        loop {
            match self.parse_pipeline_segment() {
                // Continually add segments until there is no more input or th
                Ok(segment) => {
                    segments.push(segment);

                    if self.tokens.next_if_eq(TokenContents::Pipe).is_none() {
                        // Legacy pipelines end when there are no more pipes.
                        break;
                    } else {
                        self.tokens.next_if_eq(TokenContents::Eol);
                    }
                }

                // A legacy pipeline is automatically terminated at the end of input.
                Err(ParseError::UnexpectedEof) => break,

                // Other parser errors must be returned and handled elsewhere.
                Err(error) => return Err(error),
            }
        }

        let is_async = self.tokens.next_if_eq(TokenContents::Amp).is_some();

        Ok(Pipeline { is_async, segments })
    }

    /// Parses a "smart" [`Pipeline`] with an explicit start and end.
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

            // Potentially skip a newline.
            self.tokens.next_if_eq(TokenContents::Eol);

            match self.tokens.peek().contents {
                TokenContents::Pipe => {
                    self.tokens.next();

                    // Potentially skip a newline.
                    self.tokens.next_if_eq(TokenContents::Eol);
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

        // A pipeline is only valid if it contains one or more segments.
        if segments.is_empty() {
            return Err(self.unexpected_token());
        }

        Ok(Pipeline { is_async, segments })
    }

    /// Parses a pipeline segment consisting of a [`Command`].
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

    /// Parses a sequence of [`Redirect`] definitions.
    /// Returns [`Vec::new()`] if the next non-trivial tokens are not valid redirects.
    fn parse_redirects(&mut self) -> Vec<Redirect> {
        let mut redirects = Vec::new();
        while let Ok(redirect) = self.parse_redirect() {
            redirects.push(redirect);
        }
        redirects
    }

    /// Parses a single redirect.
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

    /// Parses an interpolation consisting of multiple interpolation units.
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

    /// Parses a single interpolation unit.
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

/// Parse errors are returned by a parser when input cannot be parsed.
///
/// Note that some parse errors are recoverable, and that some errors may expected withing certain
/// contexts.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Error indicating that there is no more input to parse while parsing a started sequence.
    ///
    /// This error is recoverable, and interactive shells should prompt the user for more input.
    IncompleteSequence,

    /// Error indicating that there is no more input to parse.
    ///
    /// This error is only returned before consuming tokens in a new sequence.
    /// [`IncompleteSequence`] should instead be returned when within a sequence.
    ///
    /// This error could also mean that the input has been fully parsed.
    UnexpectedEof,

    /// Error indicating that an unexpected token was found in the input.
    /// The current sequence of tokens cannot be parsed in this context.
    ///
    /// Note that the token may still be valid in a different context.
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
