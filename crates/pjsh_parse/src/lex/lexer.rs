use std::fmt::Display;
use std::iter::Peekable;
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use crate::lex::input::{is_newline, is_variable_char, is_whitespace};
use crate::tokens::TokenContents::*;
use crate::tokens::{InterpolationUnit, TokenContents};

/// Character representing the end of input (also known as end of file = EOF).
const EOF_CHAR: char = '\0';
const EOF: &str = "\0";
type Input<'a> = Peekable<GraphemeIndices<'a>>;
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

    debug_assert_eq!(lexer.input.peek(), None, "the input should be consumed");

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
    eof: (usize, &'a str),
    input: Input<'a>,
    input_length: usize,
    mode: LexerMode<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let input = src.grapheme_indices(true).peekable();
        let eof = (src.len(), EOF);
        Self {
            eof,
            input,
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
        match self.input.peek().unwrap_or(&self.eof).1 {
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
        match self.input.peek().unwrap_or(&self.eof).1 {
            EOF => Err(LexError::UnexpectedEof),
            "\\" => {
                let (start, start_str) = self.input.next().unwrap_or(self.eof);
                if let Ok(next) = self.skip_char(delimiter) {
                    return Ok(Token::new(Quoted(next.1), Span::new(start, start + 2)));
                }
                Ok(Token::new(
                    Quoted(start_str.to_string()),
                    Span::new(start, start + 1),
                ))
            }
            ch if ch == delimiter => {
                self.mode = LexerMode::Unquoted;
                self.eat_char(Quote)
            }
            _ => {
                let (span, contents) = self.eat_while(is_quoted);
                Ok(Token::new(Quoted(contents), span))
            }
        }
    }

    /// Returns the next token in qouted multiline mode.
    fn next_quoted_multiline_token(&mut self, delimiter: &str) -> LexResult<'a> {
        debug_assert_eq!(self.mode, LexerMode::QuotedMultiline(delimiter));
        let is_quoted = |ch: &str| ch != delimiter && ch != "\\";
        match self.input.peek().unwrap_or(&self.eof).1 {
            EOF => Err(LexError::UnexpectedEof),
            ch if ch == delimiter => {
                // Use lookahead to determine whether or not a single quote or triple quotes are used.
                let (start, start_char) = self.input.next().expect("should exist");
                let mut input_iter = self.input.clone();
                let peek = [
                    input_iter.next().unwrap_or(self.eof).1,
                    input_iter.next().unwrap_or(self.eof).1,
                ];
                if peek == [delimiter, delimiter] {
                    self.input.next();
                    let end = self.input.next().unwrap_or(self.eof).0;
                    let span = Span::new(start, end + 1);
                    self.mode = LexerMode::Unquoted;
                    return Ok(Token::new(TripleQuote, span));
                }

                // Single quote, is inside the delimited value.
                Ok(Token::new(
                    Quoted(start_char.to_string()),
                    Span::new(start, start + 1),
                ))
            }
            _ => {
                let (span, contents) = self.eat_while(is_quoted);
                Ok(Token::new(Quoted(contents), span))
            }
        }
    }

    /// Eats a single character.
    fn eat_char(&mut self, contents: TokenContents) -> LexResult<'a> {
        let (index, _) = self.input.next().unwrap();
        Ok(Token::new(contents, Span::new(index, index + 1)))
    }

    /// Eats [`FileAppend`] ">>" or [`FileWrite`] ">".
    fn eat_file_write_or_append(&mut self) -> LexResult<'a> {
        let first = self.skip_char(">")?;
        if let Ok(second) = self.skip_char(">") {
            Ok(Token::new(
                FdAppendFrom(1),
                Span::new(first.0.start, second.0.end),
            ))
        } else {
            Ok(Token::new(
                FdWriteFrom(1),
                Span::new(first.0.start, first.0.end),
            ))
        }
    }

    fn eat_amp_or_andif(&mut self) -> LexResult<'a> {
        let start = *self.input.peek().unwrap_or(&self.eof);
        debug_assert_eq!(start.1, "&");
        self.input.next();

        if let Some("&") = self.input.peek().map(|&(_, c)| c) {
            let end = self.input.next().unwrap_or(self.eof).0;
            Ok(Token::new(AndIf, Span::new(start.0, end + 1)))
        } else {
            Ok(Token::new(Amp, Span::new(start.0, start.0 + 1)))
        }
    }

    fn eat_pipe_or_orif(&mut self) -> LexResult<'a> {
        let start = *self.input.peek().unwrap_or(&self.eof);
        debug_assert_eq!(start.1, "|");
        self.input.next();

        let next = self.input.peek().map(|&(_, c)| c);

        if let Some("|") = next {
            let end = self.input.next().unwrap_or(self.eof).0;
            Ok(Token::new(OrIf, Span::new(start.0, end + 1)))
        } else {
            Ok(Token::new(Pipe, Span::new(start.0, start.0 + 1)))
        }
    }

    fn eat_pipeline_start_or_literal(&mut self) -> LexResult<'a> {
        let start = *self.input.peek().unwrap_or(&self.eof);
        debug_assert_eq!(start.1, "-");

        // Use lookahead to determine whether or not to return a pipeline or a literal.
        let mut input_iter = self.input.clone();
        let peek = [
            input_iter.next().unwrap_or(self.eof).1,
            input_iter.next().unwrap_or(self.eof).1,
            input_iter.next().unwrap_or(self.eof).1,
        ];
        if peek == ["-", ">", "|"] {
            self.input = input_iter;
            return Ok(Token::new(
                PipeStart,
                Span::new(start.0, self.input.peek().unwrap_or(&self.eof).0),
            ));
        }

        self.eat_literal()
    }

    /// Eats a comment.
    fn eat_comment(&mut self) -> LexResult<'a> {
        let (span, _) = self.eat_while(|c| !is_newline(c));
        Ok(Token::new(Comment, span))
    }

    /// Eats a newline token.
    fn eat_newline(&mut self) -> LexResult<'a> {
        let start = self.input.peek().unwrap_or(&self.eof).0;
        match self.input.peek().unwrap_or(&self.eof).1 {
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
        let (span, content) = self.eat_while(|c| !is_whitespace(c));
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
        match self.input.peek().unwrap_or(&self.eof).1 {
            "{" => {
                let start = self.skip_char("{")?.0.start;
                let (mut span, content) = self.eat_while(|c| c != "}");

                let next = self.input.peek().unwrap_or(&self.eof);
                if next.1 != "}" {
                    return Err(LexError::UnexpectedChar(next.1.to_string()));
                }
                span.start = start;
                span.end = self.input.next().unwrap_or(self.eof).0 + 1;

                Ok(Token::new(Variable(content), span))
            }
            ch if ch.chars().all(char::is_alphabetic) || ch == "_" => {
                let (span, content) =
                    self.eat_while(|c| c.chars().all(char::is_alphanumeric) || c == "_");
                Ok(Token::new(Variable(content), span))
            }
            ch => Err(LexError::UnexpectedChar(ch.to_string())),
        }
    }

    fn eat_interpolation_or_variable(&mut self) -> LexResult<'a> {
        debug_assert!(self.input.peek().unwrap().1 == "$");
        let span_start = self.input.next().unwrap_or(self.eof).0;

        let result = match self.input.peek().unwrap_or(&self.eof).1 {
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
        let start = self.input.peek().unwrap_or(&self.eof).0;
        if delimiter.is_some() {
            debug_assert!(self.input.peek().unwrap().1 == delimiter.unwrap());
            self.input.next();
        }
        let mut units = Vec::new();

        loop {
            match self.input.peek().unwrap_or(&self.eof).1 {
                EOF if delimiter.is_some() => return Err(LexError::UnexpectedEof),

                // Uses EOF as default and must be matched after an actual EOF.
                ch if ch == delimiter_char => {
                    let end = self.input.next().unwrap_or(self.eof).0;
                    let span = Span::new(start, end + 1);
                    return Ok(Token::new(Interpolation(units), span));
                }
                "\\" => {
                    self.input.next();

                    if self.input.peek().unwrap_or(&self.eof).1 == "e" {
                        units.push(InterpolationUnit::Unicode('\u{001b}'));
                        self.input.next();
                        continue;
                    } else if self.skip_char("u").is_ok() {
                        if self.input.peek().unwrap_or(&self.eof).1 != "{" {
                            return Err(LexError::UnexpectedChar(
                                self.input.peek().unwrap_or(&self.eof).1.to_string(),
                            ));
                        }
                        self.input.next();

                        let content = self.eat_while(|c| c != "}").1;

                        if self.input.peek().unwrap_or(&self.eof).1 != "}" {
                            return Err(LexError::UnexpectedChar(
                                self.input.peek().unwrap_or(&self.eof).1.to_string(),
                            ));
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

                    let (_, span_str) = self.input.next().unwrap();
                    units.push(InterpolationUnit::Literal(span_str.to_string()));
                }
                "$" => {
                    self.input.next();
                    let (_, content) =
                        self.eat_while(|c| is_variable_char(c) && c != delimiter_char);
                    units.push(InterpolationUnit::Variable(content));
                }
                _ => {
                    let (_, content) =
                        self.eat_while(|c| c != "$" && c != "\\" && c != delimiter_char);
                    units.push(InterpolationUnit::Literal(content));
                }
            }
        }
    }

    /// Eats a string surrounded by quotes.
    fn eat_quoted_string(&mut self, delimiter: &'a str) -> LexResult<'a> {
        self.mode = LexerMode::Quoted(delimiter);
        let first_quote = self.eat_char(Quote);

        // Use lookahead to determine whether or not a single quote or triple quotes are used.
        let mut input_iter = self.input.clone();
        let peek = [
            input_iter.next().unwrap_or(self.eof).1,
            input_iter.next().unwrap_or(self.eof).1,
        ];
        if peek == [delimiter, delimiter] {
            self.input.next();
            let end = self.input.next().unwrap_or(self.eof);
            let span = Span::new(first_quote.expect("should exist").span.start, end.0 + 1);
            self.mode = LexerMode::QuotedMultiline(delimiter);
            return Ok(Token::new(TripleQuote, span));
        }

        first_quote
    }

    /// Eats whitespace characters.
    fn eat_whitespace(&mut self) -> LexResult<'a> {
        let (span, _) = self.eat_while(is_whitespace);
        Ok(Token::new(Whitespace, span))
    }

    /// Consumes the input while a predicate holds and returns a [`Span`] denoting the consumed
    /// character indices in the original input.
    fn eat_while(&mut self, mut predicate: impl FnMut(&str) -> bool) -> (Span, String) {
        let mut content = String::new();
        let start = self.input.peek().unwrap_or(&self.eof).0;
        let mut end = start;
        loop {
            let (i, c) = *self.input.peek().unwrap_or(&self.eof);
            if c == EOF || !predicate(c) {
                break;
            }

            self.input.next();
            end = i + 1;
            content.push_str(c);
        }

        (Span::new(start, end), content)
    }

    /// Skips a character in the input.
    fn skip_char(&mut self, ch: &str) -> Result<(Span, String), LexError> {
        let peeked = self.input.peek().unwrap_or(&self.eof).1;
        if peeked != ch {
            Err(LexError::UnexpectedChar(peeked.to_string()))
        } else {
            let next = self.input.next().unwrap_or(self.eof);
            Ok((
                Span::new(next.0, self.input.peek().unwrap_or(&self.eof).0),
                next.1.to_string(),
            ))
        }
    }
}
