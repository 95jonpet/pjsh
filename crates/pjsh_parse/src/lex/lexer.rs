use std::fmt::Display;

use crate::lex::input::{is_newline, is_variable_char, is_whitespace};
use crate::tokens::TokenContents::*;
use crate::tokens::{InterpolationUnit, TokenContents};

use super::input::Input;

/// Character representing the end of input (also known as end of file = EOF).
const EOF_CHAR: char = '\0';
const EOF: &str = "\0";
type LexResult<'a> = Result<Token, LexError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(
            start <= end,
            "Span start {} cannot come after end {}",
            start,
            end
        );
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub contents: TokenContents,
    pub span: Span,
}

impl Token {
    pub fn new(contents: TokenContents, span: Span) -> Self {
        Self { contents, span }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedChar(String),
    UnexpectedEof,
    UnknownToken(String),
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnexpectedChar(c) => write!(f, "unexpected character '{}'", c),
            LexError::UnexpectedEof => write!(f, "unexpected end of file"),
            LexError::UnknownToken(token) => write!(f, "unknown token '{}'", token),
        }
    }
}

/// Lexes some input `str` and returns all tokens within the input.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(src);
    let mut tokens = Vec::new();

    loop {
        match lexer.next_token() {
            Ok(token) if token.contents == Eof => break,
            Ok(token) => tokens.push(token),
            Err(error) => return Err(error),
        }
    }

    Ok(tokens)
}

/// Lexes some input `str` for interpolation and returns all tokens within the input.
pub fn lex_interpolation(src: &str) -> Result<Token, LexError> {
    let mut lexer = Lexer::new(src);
    let interpolation = lexer.eat_interpolation(None)?;

    debug_assert_eq!(lexer.input.peek().1, EOF, "the input should be consumed");

    Ok(interpolation)
}

/// A mode of operation for a [`Lexer`].
#[derive(Debug, PartialEq)]
enum LexerMode<'a> {
    Unquoted,
    Quoted(&'a str),
    QuotedMultiline(&'a str),
}

/// A lexer takes some `str` input from a source `src` tokenizes it, returning identified tokens
/// from the original input.
///
/// Supports multiple modes through [`LexerMode`].
pub struct Lexer<'a> {
    input: Input<'a>,
    input_length: usize,
    mode: LexerMode<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            input: Input::new(src),
            input_length: src.len(),
            mode: LexerMode::Unquoted,
        }
    }

    /// Advances the cursor and returns the next delimited token.
    pub fn next_token(&mut self) -> LexResult<'a> {
        match self.mode {
            LexerMode::Unquoted => self.next_unquoted_token(),
            LexerMode::Quoted(delimiter) => self.next_quoted_token(delimiter),
            LexerMode::QuotedMultiline(delimiter) => self.next_quoted_multiline_token(delimiter),
        }
    }

    /// Returns a token denoting the end of input (commonly known as EOF = end of file).
    fn eof_token(&self) -> Token {
        Token::new(Eof, Span::new(self.input_length, self.input_length + 1))
    }

    /// Returns the next token in unqouted mode.
    fn next_unquoted_token(&mut self) -> LexResult<'a> {
        debug_assert_eq!(self.mode, LexerMode::Unquoted);
        match self.input.peek().1 {
            "#" => self.eat_comment(),
            "|" => self.eat_pipe_or_orif(),
            "&" => self.eat_amp_or_andif(),
            ";" => self.eat_char(Semi),
            "<" => self.eat_char(FdReadTo(0)),
            ">" => self.eat_file_write_or_append(),
            "(" => self.eat_char(OpenParen),
            ")" => self.eat_char(CloseParen),
            "{" => self.eat_char(OpenBrace),
            "}" => self.eat_char(CloseBrace),
            "[" => self.eat_char(OpenBracket),
            "]" => self.eat_char(CloseBracket),
            "\"" => self.eat_quoted_string("\""),
            "'" => self.eat_quoted_string("'"),
            "$" => self.eat_interpolation_or_variable(),
            ":" => self.eat_assign_or_literal(),
            "-" => self.eat_pipeline_start_or_literal(),
            c if is_newline(c) => self.eat_newline(),
            c if is_whitespace(c) => self.eat_whitespace(),
            EOF => Ok(self.eof_token()),
            _ => self.eat_literal(),
        }
    }

    /// Returns the next token in qouted mode.
    fn next_quoted_token(&mut self, delimiter: &str) -> LexResult<'a> {
        debug_assert_eq!(self.mode, LexerMode::Quoted(delimiter));
        let is_quoted = |ch: &str| ch != delimiter && ch != "\\";
        match self.input.peek().1 {
            EOF => Err(LexError::UnexpectedEof),
            "\\" => {
                let start = self.input.next().0;
                if let Some(next) = self.input.next_if_eq(delimiter) {
                    return Ok(Token::new(
                        Quoted(next.1.to_string()),
                        Span::new(start, start + 2),
                    ));
                }
                Ok(Token::new(
                    Quoted(String::from("\\")),
                    Span::new(start, start + 1),
                ))
            }
            ch if ch == delimiter => {
                self.mode = LexerMode::Unquoted;
                self.eat_char(Quote)
            }
            _ => {
                let (span, contents) = self.input.eat_while(is_quoted);
                Ok(Token::new(Quoted(contents), span))
            }
        }
    }

    /// Returns the next token in qouted multiline mode.
    fn next_quoted_multiline_token(&mut self, delimiter: &str) -> LexResult<'a> {
        debug_assert_eq!(self.mode, LexerMode::QuotedMultiline(delimiter));
        let start = self.input.peek().0;
        let mut contents = String::new();

        loop {
            if self.input.peek_n(3) == [delimiter, delimiter, delimiter] {
                if !contents.is_empty() {
                    break;
                }

                self.input
                    .take_if_eq(&[delimiter, delimiter, delimiter])
                    .expect("peeked input should match");
                let end = self.input.peek().0;
                let span = Span::new(start, end);
                self.mode = LexerMode::Unquoted;
                return Ok(Token::new(TripleQuote, span));
            }

            match self.input.peek().1 {
                EOF => return Err(LexError::UnexpectedEof),
                ch if ch == delimiter => contents.push_str(self.input.next().1),
                _ => contents.push_str(&self.input.eat_while(|ch| ch != delimiter).1),
            }
        }

        let end = self.input.peek().0;
        let span = Span::new(start, end);
        Ok(Token::new(Quoted(contents), span))
    }

    /// Eats a single character.
    fn eat_char(&mut self, contents: TokenContents) -> LexResult<'a> {
        let (index, _) = self.input.next();
        Ok(Token::new(contents, Span::new(index, index + 1)))
    }

    /// Eats [`FileAppend`] ">>" or [`FileWrite`] ">".
    fn eat_file_write_or_append(&mut self) -> LexResult<'a> {
        let start = self
            .input
            .next_if_eq(">")
            .expect("the next char of input should be '>'")
            .0;
        if let Some(second) = self.input.next_if_eq(">") {
            Ok(Token::new(FdAppendFrom(1), Span::new(start, second.0 + 1)))
        } else {
            Ok(Token::new(FdWriteFrom(1), Span::new(start, start + 1)))
        }
    }

    fn eat_amp_or_andif(&mut self) -> LexResult<'a> {
        let start = self
            .input
            .next_if_eq("&")
            .expect("the next char of input should be '&'")
            .0;

        if let Some((end, _)) = self.input.next_if_eq("&") {
            Ok(Token::new(AndIf, Span::new(start, end + 1)))
        } else {
            Ok(Token::new(Amp, Span::new(start, start + 1)))
        }
    }

    fn eat_pipe_or_orif(&mut self) -> LexResult<'a> {
        let start = self
            .input
            .next_if_eq("|")
            .expect("the next char of input should be '|'")
            .0;

        if let Some((end, _)) = self.input.next_if_eq("|") {
            Ok(Token::new(OrIf, Span::new(start, end + 1)))
        } else {
            Ok(Token::new(Pipe, Span::new(start, start + 1)))
        }
    }

    fn eat_pipeline_start_or_literal(&mut self) -> LexResult<'a> {
        if let Some(span) = self.input.take_if_eq(&["-", ">", "|"]) {
            return Ok(Token::new(PipeStart, span));
        }

        self.eat_literal()
    }

    /// Eats a comment.
    fn eat_comment(&mut self) -> LexResult<'a> {
        let (span, _) = self.input.eat_while(|c| !is_newline(c));
        Ok(Token::new(Comment, span))
    }

    /// Eats a newline token.
    fn eat_newline(&mut self) -> LexResult<'a> {
        let start = self.input.peek().0;
        match self.input.peek().1 {
            "\r\n" => {
                self.input.next();
                Ok(Token::new(Eol, Span::new(start, start + 1)))
            }
            c if is_newline(c) => {
                self.input.next();
                Ok(Token::new(Eol, Span::new(start, start + 1)))
            }
            c => Err(LexError::UnexpectedChar(c.to_string())),
        }
    }

    /// Eats literal words.
    fn eat_literal(&mut self) -> LexResult<'a> {
        let (span, content) = self.input.eat_while(|c| !is_whitespace(c));
        Ok(Token::new(Literal(content), span))
    }

    /// Eats an assign operator or a literal word.
    fn eat_assign_or_literal(&mut self) -> LexResult<'a> {
        let token = self.eat_literal()?;
        match token.contents {
            Literal(literal) if literal == ":=" => Ok(Token::new(Assign, token.span)),
            _ => Ok(token),
        }
    }

    /// Eats variable words.
    fn eat_variable(&mut self) -> LexResult<'a> {
        match self.input.peek().1 {
            "{" => {
                let start = self.input.next().0;
                let (mut span, content) = self.input.eat_while(|c| c != "}");

                let next = self.input.peek();
                if next.1 != "}" {
                    return Err(LexError::UnexpectedChar(next.1.to_string()));
                }
                span.start = start;
                span.end = self.input.next().0 + 1;

                Ok(Token::new(Variable(content), span))
            }
            ch if ch.chars().all(char::is_alphabetic) || ch == "_" => {
                let (span, content) = self
                    .input
                    .eat_while(|c| c.chars().all(char::is_alphanumeric) || c == "_");
                Ok(Token::new(Variable(content), span))
            }
            ch => Err(LexError::UnexpectedChar(ch.to_string())),
        }
    }

    fn eat_interpolation_or_variable(&mut self) -> LexResult<'a> {
        debug_assert!(self.input.peek().1 == "$");
        let span_start = self.input.next().0;

        let result = match self.input.peek().1 {
            "\"" => self.eat_interpolation(Some("\"")),
            "'" => self.eat_interpolation(Some("'")),
            _ => self.eat_variable(),
        };

        // Account for initial $ char.
        result.map(|mut token| {
            token.span = Span::new(span_start, token.span.end);
            token
        })
    }

    /// Eats an interpolation optionally surrounded by a delimiter.
    fn eat_interpolation(&mut self, delimiter: Option<&str>) -> LexResult<'a> {
        let delimiter_char = delimiter.unwrap_or(EOF);
        let start = self.input.peek().0;
        if delimiter.is_some() {
            debug_assert!(self.input.peek().1 == delimiter.unwrap());
            self.input.next();
        }
        let mut units = Vec::new();

        loop {
            match self.input.peek().1 {
                EOF if delimiter.is_some() => return Err(LexError::UnexpectedEof),

                // Uses EOF as default and must be matched after an actual EOF.
                ch if ch == delimiter_char => {
                    let end = self.input.next().0;
                    let span = Span::new(start, end + 1);
                    return Ok(Token::new(Interpolation(units), span));
                }
                "\\" => {
                    self.input.next();

                    if self.input.next_if_eq("e").is_some() {
                        units.push(InterpolationUnit::Unicode('\u{001b}'));
                        continue;
                    } else if self.input.next_if_eq("u").is_some() {
                        if self.input.peek().1 != "{" {
                            return Err(LexError::UnexpectedChar(self.input.peek().1.to_string()));
                        }
                        self.input.next();

                        let content = self.input.eat_while(|c| c != "}").1;

                        if self.input.peek().1 != "}" {
                            return Err(LexError::UnexpectedChar(self.input.peek().1.to_string()));
                        }
                        self.input.next();

                        if let Ok(code) = u32::from_str_radix(&content, 16) {
                            let ch = char::from_u32(code).unwrap_or(EOF_CHAR);
                            units.push(InterpolationUnit::Unicode(ch));
                            continue;
                        } else {
                            return Err(LexError::UnknownToken(format!("\\u{{{}}}", content)));
                        }
                    }

                    let (_, span_str) = self.input.next();
                    units.push(InterpolationUnit::Literal(span_str.to_string()));
                }
                "$" => {
                    self.input.next();
                    let (_, content) = self
                        .input
                        .eat_while(|c| is_variable_char(c) && c != delimiter_char);
                    units.push(InterpolationUnit::Variable(content));
                }
                _ => {
                    let (_, content) = self
                        .input
                        .eat_while(|c| c != "$" && c != "\\" && c != delimiter_char);
                    units.push(InterpolationUnit::Literal(content));
                }
            }
        }
    }

    /// Eats a string surrounded by quotes.
    fn eat_quoted_string(&mut self, delimiter: &'a str) -> LexResult<'a> {
        self.mode = LexerMode::Quoted(delimiter);
        let first_quote = self.eat_char(Quote);

        // Peek the next two quotes to determine whether a single quote or triple quotes are used.
        if let Some(end_span) = self.input.take_if_eq(&[delimiter, delimiter]) {
            let span = Span::new(first_quote.expect("should exist").span.start, end_span.end);
            self.mode = LexerMode::QuotedMultiline(delimiter);
            return Ok(Token::new(TripleQuote, span));
        }

        first_quote
    }

    /// Eats whitespace characters.
    fn eat_whitespace(&mut self) -> LexResult<'a> {
        let (span, _) = self.input.eat_while(is_whitespace);
        Ok(Token::new(Whitespace, span))
    }
}