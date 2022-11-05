use std::collections::HashMap;

use crate::lex::lexer::LexError;
use crate::token::{self, Token, TokenContents};
use crate::ParseError;
use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Block, Command, ConditionalChain, ConditionalLoop, FileDescriptor,
    ForIterableLoop, Function, InterpolationUnit, Iterable, List, Pipeline, PipelineSegment,
    Program, Redirect, RedirectMode, Statement, Word,
};

use super::cursor::TokenCursor;
use super::iterable::parse_iterable;

/// Tries to parse a [`Program`] by consuming some input `src` in its entirety.
/// A [`ParseError`] is returned if a program can't be parsed.
pub fn parse(src: &str, aliases: &HashMap<String, String>) -> Result<Program, ParseError> {
    match crate::lex(src, aliases) {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            parser.parse_program()
        }
        Err(LexError::UnexpectedEof) => Err(ParseError::UnexpectedEof),
        Err(error) => Err(ParseError::InvalidSyntax(error.to_string())),
    }
}

/// Tries to parse a [`Word`] from within an interpolation.
/// A [`ParseError`] is returned if a program can't be parsed.
pub fn parse_interpolation(src: &str) -> Result<Word, ParseError> {
    match crate::lex_interpolation(src) {
        Ok(token) => {
            let mut parser = Parser::new(vec![token]);
            parser.parse_word()
        }
        Err(LexError::UnexpectedEof) => Err(ParseError::UnexpectedEof),
        Err(error) => Err(ParseError::InvalidSyntax(error.to_string())),
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

    /// Parses a non-empty subshell.
    ///
    /// Note that tokens are consumed as long as the subshell is opened - even if the subshell is
    /// empty.
    fn parse_subshell(&mut self) -> Result<Statement, ParseError> {
        if self.tokens.next_if_eq(TokenContents::OpenParen).is_none() {
            return Err(self.unexpected_token());
        }

        let subshell_program = self.parse_subshell_program()?;

        // A subshell must be terminated by a closing parenthesis.
        if self.tokens.next_if_eq(TokenContents::CloseParen).is_none() {
            return Err(ParseError::IncompleteSequence);
        }

        // A subshell must not be empty.
        if subshell_program.statements.is_empty() {
            return Err(ParseError::EmptySubshell);
        }

        Ok(Statement::Subshell(subshell_program))
    }

    fn parse_subshell_program(&mut self) -> Result<Program, ParseError> {
        let mut subshell_program = Program::new();
        loop {
            match self.parse_statement() {
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

    /// Parses an [`AndOr`] consisting of one or more [`Pipeline`] definitions.
    pub fn parse_and_or(&mut self) -> Result<AndOr, ParseError> {
        let mut and_or = AndOr::default();
        and_or.pipelines.push(self.parse_pipeline()?);

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

            and_or.operators.push(operator);
            and_or.pipelines.push(self.parse_pipeline()?);
        }

        Ok(and_or)
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
        let mut pipeline = Pipeline::default();

        // No input to parse - a valid legacy pipeline cannot be constructed.
        if self.tokens.peek().contents == TokenContents::Eof {
            return Err(ParseError::UnexpectedEof);
        }

        loop {
            match self.parse_pipeline_segment() {
                // Continually add segments until there is no more input.
                Ok(segment) => {
                    pipeline.segments.push(segment);

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

        pipeline.is_async = self.tokens.next_if_eq(TokenContents::Amp).is_some();

        Ok(pipeline)
    }

    /// Parses a "smart" [`Pipeline`] with an explicit start and end.
    pub fn parse_smart_pipeline(&mut self) -> Result<Pipeline, ParseError> {
        self.tokens.newline_is_whitespace(true); // Newline is trivialized in a smart pipeline.
        let mut pipeline = Pipeline::default();

        loop {
            match self.tokens.peek().contents {
                TokenContents::Amp => {
                    self.tokens.next();
                    pipeline.is_async = true;
                    break;
                }
                TokenContents::Semi => {
                    self.tokens.next();
                    break;
                }
                _ => pipeline.segments.push(self.parse_pipeline_segment()?),
            }

            match self.tokens.peek().contents {
                TokenContents::Pipe => {
                    self.tokens.next();
                }
                TokenContents::Eof => return Err(ParseError::IncompleteSequence),
                TokenContents::Amp => {
                    self.tokens.next();
                    pipeline.is_async = true;
                    break;
                }
                TokenContents::Semi => {
                    self.tokens.next();
                    break;
                }
                _ => return Err(self.unexpected_token()),
            }
        }

        self.tokens.newline_is_whitespace(false); // Ensure a clean exit.

        // A pipeline is only valid if it contains one or more segments.
        if pipeline.segments.is_empty() {
            return Err(self.unexpected_token());
        }

        Ok(pipeline)
    }

    /// Parses a pipeline segment.
    pub fn parse_pipeline_segment(&mut self) -> Result<PipelineSegment, ParseError> {
        if let Ok(condition) = self.parse_condition() {
            return Ok(PipelineSegment::Condition(condition));
        }

        Ok(PipelineSegment::Command(self.parse_command()?))
    }

    /// Tries to parse a [`Condition`] from the next tokens of input.
    pub fn parse_condition(&mut self) -> Result<Vec<Word>, ParseError> {
        if self
            .tokens
            .next_if_eq(TokenContents::DoubleOpenBracket)
            .is_none()
        {
            return Err(self.unexpected_token());
        }

        let mut words = Vec::new();
        while let Ok(word) = self.parse_word() {
            words.push(word);
        }

        match self.tokens.peek().contents {
            TokenContents::DoubleCloseBracket => self.tokens.next(),
            TokenContents::Eol => return Err(ParseError::IncompleteSequence),
            _ => return Err(self.unexpected_token()),
        };

        Ok(words)
    }

    /// Tries to parse a [`Command`] from the next tokens of input.
    pub fn parse_command(&mut self) -> Result<Command, ParseError> {
        let prefix_redirects = self.parse_redirects();
        let mut command = Command::default();
        command.arg(self.parse_word()?);

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
        self.tokens.newline_is_whitespace(false); // Ensure clean start.
        self.skip_newlines();

        // Try to parse a subshell.
        match self.parse_subshell() {
            Ok(subshell_statement) => return Ok(subshell_statement),
            Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
            _ => (),
        }

        // Try to parse an if-statement.
        match self.parse_if_statement() {
            Ok(statement) => return Ok(statement),
            Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
            _ => (),
        }

        // Try to parse a for-in-loops.
        match self.parse_for_in_loop() {
            Ok(statement) => return Ok(statement),
            Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
            _ => (),
        }

        // Try to parse a while-loop.
        match self.parse_while_loop() {
            Ok(statement) => return Ok(statement),
            Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
            _ => (),
        }

        // Try to parse a function declaration.
        match self.parse_function() {
            Ok(function_statement) => return Ok(function_statement),
            Err(ParseError::IncompleteSequence) => return Err(ParseError::IncompleteSequence),
            _ => (),
        }

        // Try to parse an assignment.
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
            TokenContents::ProcessSubstitutionStart => self.parse_process_substitution(),
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
                    FileDescriptor::Number(fd),
                    RedirectMode::Write,
                ))
            }
            TokenContents::FdWriteFrom(fd) => {
                self.tokens.next();
                Ok(Redirect::new(
                    FileDescriptor::Number(fd),
                    FileDescriptor::File(self.parse_word()?),
                    RedirectMode::Write,
                ))
            }
            TokenContents::FdAppendFrom(fd) => {
                self.tokens.next();
                Ok(Redirect::new(
                    FileDescriptor::Number(fd),
                    FileDescriptor::File(self.parse_word()?),
                    RedirectMode::Append,
                ))
            }
            _ => Err(self.unexpected_token()),
        }
    }

    /// Parses a process substitution.
    fn parse_process_substitution(&mut self) -> Result<Word, ParseError> {
        self.tokens.next();

        let program = self.parse_subshell_program()?;

        if self.tokens.next_if_eq(TokenContents::CloseParen).is_none() {
            return Err(ParseError::IncompleteSequence);
        }

        Ok(Word::ProcessSubstitution(program))
    }

    /// Parses an interpolation consisting of multiple interpolation units.
    fn parse_interpolation(&mut self) -> Result<Word, ParseError> {
        if let TokenContents::Interpolation(units) = self.tokens.next().contents {
            let mut word_units = Vec::with_capacity(units.len());
            for unit in units {
                word_units.push(self.parse_interpolation_unit(unit)?);
            }
            Ok(Word::Interpolation(word_units))
        } else {
            Err(self.unexpected_token())
        }
    }

    /// Parses a single interpolation unit.
    fn parse_interpolation_unit(
        &self,
        unit: token::InterpolationUnit,
    ) -> Result<InterpolationUnit, ParseError> {
        match unit {
            token::InterpolationUnit::Literal(literal) => Ok(InterpolationUnit::Literal(literal)),
            token::InterpolationUnit::Unicode(ch) => Ok(InterpolationUnit::Unicode(ch)),
            token::InterpolationUnit::Variable(var) => Ok(InterpolationUnit::Variable(var)),
            token::InterpolationUnit::Subshell(subshell_tokens) => {
                let mut subshell_parser = Parser::new(subshell_tokens);
                let subshell_program = subshell_parser.parse_program()?;
                Ok(InterpolationUnit::Subshell(subshell_program))
            }
        }
    }

    /// Parses a triple quoted word.
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

        if !matches!(lines.next(), Some(first_line) if first_line.is_empty()) {
            return Err(ParseError::InvalidSyntax(
                "multiline strings must contain at least one line".to_owned(),
            ));
        }

        if let Some(line) = lines.next() {
            let trimmed = line.trim_start_matches(char::is_whitespace);
            indent = line.len() - trimmed.len();
            string.push_str(trimmed);
        }

        for line in lines {
            let prefix = &line[..indent];
            if let Some((i, ch)) = prefix.char_indices().find(|(_, ch)| !ch.is_whitespace()) {
                return Err(ParseError::InvalidSyntax(format!(
                    "expected an indentation of {indent}, found {ch} at character {i}"
                )));
            }

            string.push('\n');
            string.push_str(&line[indent..]);
        }

        Ok(Word::Quoted(string))
    }

    /// Parses a quoted word.
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

    /// Parses a function declaration,
    fn parse_function(&mut self) -> Result<Statement, ParseError> {
        if self
            .tokens
            .next_if_eq(TokenContents::Literal("fn".into()))
            .is_none()
        {
            return Err(self.unexpected_token());
        }

        match self.tokens.next().contents {
            TokenContents::Literal(name) => {
                if self.tokens.next_if_eq(TokenContents::OpenParen).is_none() {
                    return Err(self.unexpected_token());
                }

                // Parse argument list.
                let mut args = Vec::new();
                while let Some(token) = self
                    .tokens
                    .next_if(|t| matches!(&t.contents, &TokenContents::Literal(_)))
                {
                    match token.contents {
                        TokenContents::Literal(arg) => args.push(arg),
                        _ => unreachable!(),
                    };
                }

                if self.tokens.next_if_eq(TokenContents::CloseParen).is_none() {
                    return Err(self.unexpected_token());
                }

                Ok(Statement::Function(Function::new(
                    name,
                    args,
                    self.parse_block()?,
                )))
            }
            _ => Err(self.unexpected_token()),
        }
    }

    /// Parses an if-statement.
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        if self
            .tokens
            .next_if_eq(TokenContents::Literal("if".into()))
            .is_none()
        {
            return Err(self.unexpected_token());
        }

        // Parse the initial condition and branch.
        let mut conditions = vec![self.parse_and_or()?];
        let mut branches = vec![self.parse_block()?];

        loop {
            if self
                .tokens
                .next_if_eq(TokenContents::Literal("else".into()))
                .is_none()
            {
                break;
            }

            if self
                .tokens
                .next_if_eq(TokenContents::Literal("if".into()))
                .is_some()
            {
                conditions.push(self.parse_and_or()?);
                branches.push(self.parse_block()?);
                continue;
            }

            branches.push(self.parse_block()?);
            break;
        }

        Ok(Statement::If(ConditionalChain {
            conditions,
            branches,
        }))
    }

    /// Parses a for-in-loop.
    pub(crate) fn parse_for_in_loop(&mut self) -> Result<Statement, ParseError> {
        if self
            .tokens
            .next_if_eq(TokenContents::Literal("for".into()))
            .is_none()
        {
            return Err(self.unexpected_token());
        }

        let variable = match self.parse_word() {
            Ok(Word::Literal(literal)) => literal,
            Ok(_) => return Err(ParseError::InvalidSyntax("expected literal".to_owned())),
            Err(error) => return Err(error),
        };

        if self
            .tokens
            .next_if_eq(TokenContents::Literal("in".into()))
            .is_none()
        {
            return Err(self.unexpected_token());
        }

        let iterable = if let Ok(list) = self.parse_list() {
            Iterable::from(list)
        } else {
            match self.parse_word() {
                Ok(Word::Literal(literal)) => parse_iterable(&literal)?,
                Ok(_) => return Err(ParseError::InvalidSyntax("expected iterable".to_owned())),
                Err(error) => return Err(error),
            }
        };

        Ok(Statement::ForIn(ForIterableLoop {
            variable,
            iterable,
            body: self.parse_block()?,
        }))
    }

    /// Parses a while-loop.
    fn parse_while_loop(&mut self) -> Result<Statement, ParseError> {
        if self
            .tokens
            .next_if_eq(TokenContents::Literal("while".into()))
            .is_none()
        {
            return Err(self.unexpected_token());
        }

        Ok(Statement::While(ConditionalLoop {
            condition: self.parse_and_or()?,
            body: self.parse_block()?,
        }))
    }

    /// Parses a code block surrounded by curly braces.
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        if self.tokens.next_if_eq(TokenContents::OpenBrace).is_none() {
            return Err(self.unexpected_token());
        }
        let mut block = Block::default();
        loop {
            match &self.tokens.peek().contents {
                TokenContents::Eol => self.skip_newlines(),
                TokenContents::Eof => return Err(ParseError::IncompleteSequence),
                TokenContents::CloseBrace => break,
                _ => {
                    block.statement(self.parse_statement()?);
                }
            }
        }
        if self.tokens.next_if_eq(TokenContents::CloseBrace).is_none() {
            return Err(self.unexpected_token());
        }
        Ok(block)
    }

    /// Parses a list of words surrounded by square brackets.
    pub(crate) fn parse_list(&mut self) -> Result<List, ParseError> {
        if self.tokens.next_if_eq(TokenContents::OpenBracket).is_none() {
            return Err(self.unexpected_token());
        }
        let mut list = List::default();
        loop {
            match &self.tokens.peek().contents {
                TokenContents::Eol => self.skip_newlines(),
                TokenContents::Eof => return Err(ParseError::IncompleteSequence),
                TokenContents::CloseBracket => break,
                _ => {
                    list.push(self.parse_word()?);
                }
            }
        }
        if self
            .tokens
            .next_if_eq(TokenContents::CloseBracket)
            .is_none()
        {
            return Err(self.unexpected_token());
        }
        Ok(list)
    }

    /// Advances the token cursor until the next token is not an end-of-line token.
    fn skip_newlines(&mut self) {
        while matches!(
            self.tokens.peek().contents,
            TokenContents::Eol | TokenContents::Semi
        ) {
            self.tokens.next();
        }
    }

    /// Returns a [`ParseError::UnexpectedToken`] around a copy of the next token.
    fn unexpected_token(&mut self) -> ParseError {
        ParseError::UnexpectedToken(self.tokens.peek().clone())
    }
}
