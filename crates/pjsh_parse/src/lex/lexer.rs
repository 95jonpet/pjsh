use std::fmt::Display;
use std::iter::Peekable;
use std::str::CharIndices;

use crate::lex::input::{is_newline, is_variable_char, is_whitespace};
use crate::tokens::TokenContents::*;
use crate::tokens::{InterpolationUnit, TokenContents};

/// Character representing the end of input (also known as end of file = EOF).
const EOF_CHAR: char = '\0';
type Input<'a> = Peekable<CharIndices<'a>>;
type LexResult<'a> = Result<Token<'a>, LexError>;

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
pub struct Token<'a> {
    pub contents: TokenContents<'a>,
    pub span: Span,
}

impl<'a> Token<'a> {
    pub fn new(contents: TokenContents<'a>, span: Span) -> Self {
        Self { contents, span }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedChar(char),
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
pub fn lex(src: &str) -> Result<Vec<Token<'_>>, LexError> {
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
pub fn lex_interpolation(src: &str) -> Result<Token<'_>, LexError> {
    let mut lexer = Lexer::new(src);
    let interpolation = lexer.eat_interpolation(None)?;

    debug_assert_eq!(lexer.input.peek(), None, "the input should be consumed");

    Ok(interpolation)
}

/// A mode of operation for a [`Lexer`].
#[derive(Debug, PartialEq)]
enum LexerMode {
    Unquoted,
    Quoted(char),
    QuotedMultiline(char),
}

/// A lexer takes some `str` input from a source `src` tokenizes it, returning identified tokens
/// from the original input.
///
/// Supports multiple modes through [`LexerMode`].
pub struct Lexer<'a> {
    src: &'a str,
    input: Input<'a>,
    input_length: usize,
    mode: LexerMode,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let input = src.char_indices().peekable();
        Self {
            src,
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
    fn eof_token(&self) -> Token<'a> {
        Token::new(Eof, Span::new(self.input_length, self.input_length + 1))
    }

    /// Returns the next token in unqouted mode.
    fn next_unquoted_token(&mut self) -> LexResult<'a> {
        debug_assert_eq!(self.mode, LexerMode::Unquoted);
        let default = (self.input_length, EOF_CHAR);
        match self.input.peek().unwrap_or(&default).1 {
            '#' => self.eat_comment(),
            '|' => self.eat_pipe_or_orif(),
            '&' => self.eat_amp_or_andif(),
            ';' => self.eat_char(Semi),
            '<' => self.eat_char(FileRead),
            '>' => self.eat_file_write_or_append(),
            '(' => self.eat_char(OpenParen),
            ')' => self.eat_char(CloseParen),
            '{' => self.eat_char(OpenBrace),
            '}' => self.eat_char(CloseBrace),
            '[' => self.eat_char(OpenBracket),
            ']' => self.eat_char(CloseBracket),
            '"' => self.eat_quoted_string('"'),
            '\'' => self.eat_quoted_string('\''),
            '$' => self.eat_interpolation_or_variable(),
            ':' => self.eat_assign_or_literal(),
            '-' => self.eat_pipeline_start_or_literal(),
            c if is_newline(&c) => self.eat_newline(),
            c if is_whitespace(&c) => self.eat_whitespace(),
            EOF_CHAR => Ok(self.eof_token()),
            _ => self.eat_literal(),
        }
    }

    /// Returns the next token in qouted mode.
    fn next_quoted_token(&mut self, delimiter: char) -> LexResult<'a> {
        debug_assert_eq!(self.mode, LexerMode::Quoted(delimiter));
        let default = (self.input_length, EOF_CHAR);
        let is_quoted = |ch: &char| ch != &delimiter && ch != &'\\';
        match self.input.peek().unwrap_or(&default).1 {
            EOF_CHAR => Err(LexError::UnexpectedEof),
            '\\' => {
                let start = self.input.next().unwrap_or(default).0;
                if self.skip_char(delimiter).is_ok() {
                    return Ok(Token::new(
                        Quoted(&self.src[start + 1..start + 2]),
                        Span::new(start, start + 2),
                    ));
                }
                Ok(Token::new(
                    Quoted(&self.src[start..start + 1]),
                    Span::new(start, start + 1),
                ))
            }
            ch if ch == delimiter => {
                self.mode = LexerMode::Unquoted;
                self.eat_char(Quote)
            }
            _ => {
                let span = self.eat_while(is_quoted);
                let contents = &self.src[span.start..span.end];
                Ok(Token::new(Quoted(contents), span))
            }
        }
    }

    /// Returns the next token in qouted multiline mode.
    fn next_quoted_multiline_token(&mut self, delimiter: char) -> LexResult<'a> {
        debug_assert_eq!(self.mode, LexerMode::QuotedMultiline(delimiter));
        let default = (self.input_length, EOF_CHAR);
        let is_quoted = |ch: &char| ch != &delimiter && ch != &'\\';
        match self.input.peek().unwrap_or(&default).1 {
            EOF_CHAR => Err(LexError::UnexpectedEof),
            ch if ch == delimiter => {
                // Use lookahead to determine whether or not a single quote or triple quotes are used.
                let start = self.input.next().expect("should exist").0;
                let mut input_iter = self.input.clone();
                let peek = [
                    input_iter.next().unwrap_or(default).1,
                    input_iter.next().unwrap_or(default).1,
                ];
                if peek == [delimiter, delimiter] {
                    self.input.next();
                    let end = self.input.next().unwrap_or(default).0;
                    let span = Span::new(start, end + 1);
                    self.mode = LexerMode::Unquoted;
                    return Ok(Token::new(TripleQuote, span));
                }

                // Single quote, is inside the delimited value.
                return Ok(Token::new(
                    Quoted(&self.src[start..start + 1]),
                    Span::new(start, start + 1),
                ));
            }
            _ => {
                let span = self.eat_while(is_quoted);
                let contents = &self.src[span.start..span.end];
                return Ok(Token::new(Quoted(contents), span));
            }
        }
    }

    /// Eats a single character.
    fn eat_char(&mut self, contents: TokenContents<'a>) -> LexResult<'a> {
        let (index, _) = self.input.next().unwrap();
        Ok(Token::new(contents, Span::new(index, index + 1)))
    }

    /// Eats [`FileAppend`] ">>" or [`FileWrite`] ">".
    fn eat_file_write_or_append(&mut self) -> LexResult<'a> {
        let first = self.skip_char('>')?;
        if let Ok(second) = self.skip_char('>') {
            Ok(Token::new(FileAppend, Span::new(first.start, second.end)))
        } else {
            Ok(Token::new(FileWrite, Span::new(first.start, first.end)))
        }
    }

    fn eat_amp_or_andif(&mut self) -> LexResult<'a> {
        let default = (self.input_length, EOF_CHAR);
        let start = *self.input.peek().unwrap_or(&default);
        debug_assert_eq!(start.1, '&');
        self.input.next();

        if let Some('&') = self.input.peek().map(|&(_, c)| c) {
            let end = self.input.next().unwrap_or(default).0;
            Ok(Token::new(AndIf, Span::new(start.0, end + 1)))
        } else {
            Ok(Token::new(Amp, Span::new(start.0, start.0 + 1)))
        }
    }

    fn eat_pipe_or_orif(&mut self) -> LexResult<'a> {
        let default = (self.input_length, EOF_CHAR);
        let start = *self.input.peek().unwrap_or(&default);
        debug_assert_eq!(start.1, '|');
        self.input.next();

        let next = self.input.peek().map(|&(_, c)| c);

        if let Some('|') = next {
            let end = self.input.next().unwrap_or(default).0;
            Ok(Token::new(OrIf, Span::new(start.0, end + 1)))
        } else {
            Ok(Token::new(Pipe, Span::new(start.0, start.0 + 1)))
        }
    }

    fn eat_pipeline_start_or_literal(&mut self) -> LexResult<'a> {
        let default = (self.input_length, EOF_CHAR);
        let start = *self.input.peek().unwrap_or(&default);
        debug_assert_eq!(start.1, '-');
        self.input.next();

        if self.input.peek().unwrap_or(&default).1 != '>' {
            return self.eat_literal().map(|token| {
                let span = Span::new(token.span.start - 1, token.span.end);
                let contents = &self.src[span.start..span.end];
                Token::new(Literal(contents), span)
            });
        }
        self.input.next();

        if self.input.peek().unwrap_or(&default).1 != '|' {
            return self.eat_literal().map(|token| {
                let span = Span::new(token.span.start - 2, token.span.end);
                let contents = &self.src[span.start..span.end];
                Token::new(Literal(contents), span)
            });
        }
        let end = self.input.next().unwrap_or(default).0;

        Ok(Token::new(PipeStart, Span::new(start.0, end)))
    }

    /// Eats a comment.
    fn eat_comment(&mut self) -> LexResult<'a> {
        let span = self.eat_while(|c| !is_newline(c));
        Ok(Token::new(Comment, span))
    }

    /// Eats a newline token.
    fn eat_newline(&mut self) -> LexResult<'a> {
        let default = (self.input_length, EOF_CHAR);
        let start = self.input.peek().unwrap_or(&default).0;
        match self.input.peek().unwrap_or(&default).1 {
            '\r' => {
                self.input.next();
                if let (_, '\n') = self.input.peek().unwrap_or(&default) {
                    self.input.next();
                    Ok(Token::new(Eol, Span::new(start, start + 2)))
                } else {
                    Ok(Token::new(Eol, Span::new(start, start + 1)))
                }
            }
            c if is_newline(&c) => {
                self.input.next();
                Ok(Token::new(Eol, Span::new(start, start + 1)))
            }
            c => Err(LexError::UnexpectedChar(c)),
        }
    }

    /// Eats literal words.
    fn eat_literal(&mut self) -> LexResult<'a> {
        let span = self.eat_while(|c| !is_whitespace(c));
        Ok(Token::new(Literal(&self.src[span.start..span.end]), span))
    }

    /// Eats an assign operator or a literal word.
    fn eat_assign_or_literal(&mut self) -> LexResult<'a> {
        let token = self.eat_literal()?;
        match token.contents {
            Literal(":=") => Ok(Token::new(Assign, token.span)),
            _ => Ok(token),
        }
    }

    /// Eats variable words.
    fn eat_variable(&mut self) -> LexResult<'a> {
        let default = (self.input_length, EOF_CHAR);

        match self.input.peek().unwrap_or(&default).1 {
            '{' => {
                let mut span = self.eat_while(|c| c != &'}');

                let next = self.input.peek().unwrap_or(&default);
                if next.1 != '}' {
                    return Err(LexError::UnexpectedChar(next.1));
                }
                span.end = self.input.next().unwrap_or(default).0 + 1;

                Ok(Token::new(
                    Variable(&self.src[span.start + 1..span.end - 1]),
                    span,
                ))
            }
            ch if ch.is_alphabetic() || ch == '_' => {
                let span = self.eat_while(|c| c.is_alphanumeric() || c == &'_');
                Ok(Token::new(Variable(&self.src[span.start..span.end]), span))
            }
            ch => Err(LexError::UnexpectedChar(ch)),
        }
    }

    fn eat_interpolation_or_variable(&mut self) -> LexResult<'a> {
        debug_assert!(self.input.peek().unwrap().1 == '$');
        let default = (self.input_length, EOF_CHAR);
        let span_start = self.input.next().unwrap_or(default).0;

        let result = match self.input.peek().unwrap_or(&default).1 {
            '"' => self.eat_interpolation(Some('"')),
            '\'' => self.eat_interpolation(Some('\'')),
            _ => self.eat_variable(),
        };

        // Account for initial $ char.
        result.map(|mut token| {
            token.span = Span::new(span_start, token.span.end);
            token
        })
    }

    /// Eats an interpolation optionally surrounded by a delimiter.
    fn eat_interpolation(&mut self, delimiter: Option<char>) -> LexResult<'a> {
        let delimiter_char = delimiter.unwrap_or(EOF_CHAR);
        let default = (self.input_length, EOF_CHAR);
        let start = self.input.peek().unwrap_or(&default).0;
        if delimiter.is_some() {
            debug_assert!(self.input.peek().unwrap().1 == delimiter.unwrap());
            self.input.next();
        }
        let mut units = Vec::new();

        loop {
            match self.input.peek().unwrap_or(&default).1 {
                EOF_CHAR if delimiter.is_some() => return Err(LexError::UnexpectedEof),

                // Uses EOF_CHAR as default and must be matched after an actual EOF_CHAR.
                ch if ch == delimiter_char => {
                    let end = self.input.next().unwrap_or(default).0;
                    let span = Span::new(start, end + 1);
                    return Ok(Token::new(Interpolation(units), span));
                }
                '\\' => {
                    self.input.next();

                    if self.input.peek().unwrap_or(&default).1 == 'e' {
                        units.push(InterpolationUnit::Unicode('\u{001b}'));
                        self.input.next();
                        continue;
                    } else if self.skip_char('u').is_ok() {
                        if self.input.peek().unwrap_or(&default).1 != '{' {
                            return Err(LexError::UnexpectedChar(
                                self.input.peek().unwrap_or(&default).1,
                            ));
                        }
                        self.input.next();

                        let span = self.eat_while(|c| c != &'}');

                        if self.input.peek().unwrap_or(&default).1 != '}' {
                            return Err(LexError::UnexpectedChar(
                                self.input.peek().unwrap_or(&default).1,
                            ));
                        }
                        self.input.next();

                        if let Ok(code) = u32::from_str_radix(&self.src[span.start..span.end], 16) {
                            let ch = char::from_u32(code).unwrap_or(EOF_CHAR);
                            units.push(InterpolationUnit::Unicode(ch));
                            continue;
                        } else {
                            return Err(LexError::UnknownToken(format!(
                                "\\u{{{}}}",
                                &self.src[span.start..span.end]
                            )));
                        }
                    }

                    let span_start = self.input.next().unwrap().0;
                    let span = Span::new(span_start, span_start + 1);
                    units.push(InterpolationUnit::Literal(&self.src[span.start..span.end]));
                }
                '$' => {
                    self.input.next();
                    let span = self.eat_while(|c| is_variable_char(c) && c != &delimiter_char);
                    units.push(InterpolationUnit::Variable(&self.src[span.start..span.end]));
                }
                _ => {
                    let span = self.eat_while(|c| c != &'$' && c != &'\\' && c != &delimiter_char);
                    units.push(InterpolationUnit::Literal(&self.src[span.start..span.end]));
                }
            }
        }
    }

    /// Eats a string surrounded by quotes.
    fn eat_quoted_string(&mut self, delimiter: char) -> LexResult<'a> {
        self.mode = LexerMode::Quoted(delimiter);
        let default = (self.input_length, EOF_CHAR);
        let first_quote = self.eat_char(Quote);

        // Use lookahead to determine whether or not a single quote or triple quotes are used.
        let mut input_iter = self.input.clone();
        let peek = [
            input_iter.next().unwrap_or(default).1,
            input_iter.next().unwrap_or(default).1,
        ];
        if peek == [delimiter, delimiter] {
            self.input.next();
            let end = self.input.next().unwrap_or(default);
            let span = Span::new(first_quote.expect("should exist").span.start, end.0 + 1);
            self.mode = LexerMode::QuotedMultiline(delimiter);
            return Ok(Token::new(TripleQuote, span));
        }

        first_quote
    }

    /// Eats whitespace characters.
    fn eat_whitespace(&mut self) -> LexResult<'a> {
        let span = self.eat_while(is_whitespace);
        Ok(Token::new(Whitespace, span))
    }

    /// Consumes the input while a predicate holds and returns a [`Span`] denoting the consumed
    /// character indices in the original input.
    fn eat_while(&mut self, mut predicate: impl FnMut(&char) -> bool) -> Span {
        let default = (self.input_length, EOF_CHAR);
        let start = self.input.peek().unwrap_or(&default).0;
        let mut end = start;
        loop {
            let (i, c) = *self.input.peek().unwrap_or(&default);
            if c == EOF_CHAR || !predicate(&c) {
                break;
            }

            self.input.next();
            end = i + 1;
        }

        Span::new(start, end)
    }

    /// Skips a char in the input.
    fn skip_char(&mut self, ch: char) -> Result<Span, LexError> {
        let default = (self.input_length, EOF_CHAR);
        let peeked = self.input.peek().unwrap_or(&default).1;
        if peeked != ch {
            Err(LexError::UnexpectedChar(peeked))
        } else {
            let start = self.input.next().unwrap_or(default).0;
            Ok(Span::new(start, start + 1))
        }
    }
}
