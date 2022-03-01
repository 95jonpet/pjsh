use std::io;

use thiserror::Error;

use crate::tok;

#[derive(Debug, Error, PartialEq)]
pub enum LexError<'a> {
    #[error("Was expecting {expected:?}, found {found:?}")]
    MissingExpectedSymbol {
        expected: TokenType<'a>,
        found: Token<'a>,
    },

    #[error("Can't find opening symbol ({open:?}) for {symbol:?}.")]
    MisbalancedSymbol { symbol: char, open: char },

    #[error("Unknown symbol '{symbol}'")]
    UnknownSymbol { symbol: String },

    #[error("Unexpected end of file: expected {expected}")]
    UnexpectedEof { expected: String },
}

pub type Token<'a> = TokenType<'a>;
type BalancingDepthType = i32;

#[derive(Debug, PartialEq)]
pub enum TokenType<'a> {
    Eof,
    Punctuation { raw: char, kind: PunctuationKind },
    Operator { raw: &'a str, kind: OperatorKind },
    Identifier(&'a str),
    String(String),
}

#[derive(Debug, PartialEq)]
pub enum OperatorKind {
    Amp,
    AndIf,
    OrIf,
    Pipe,
    PipeStart,
}

#[derive(Debug, PartialEq)]
pub enum PunctuationKind {
    Open(BalancingDepthType),
    Close(BalancingDepthType),
    Separator,
}

pub struct Lexer<'a> {
    pub cur_line: usize,
    pub cur_column: usize,

    pub codepoint_offset: usize,

    input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    balancing_state: std::collections::HashMap<char, BalancingDepthType>,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: &'a str) -> Self {
        Self {
            cur_column: 1,
            cur_line: 1,
            codepoint_offset: 0,
            input: chars,
            chars: chars.chars().peekable(),
            balancing_state: std::collections::HashMap::new(),
        }
    }

    fn map_balance(c: &char) -> char {
        match c {
            '}' => '{',
            '{' => '}',
            ']' => '[',
            '[' => ']',
            ')' => '(',
            '(' => ')',
            _ => panic!("Cannot map {c} to another char."),
        }
    }

    fn push_symbol(&mut self, c: &char) -> BalancingDepthType {
        if let Some(v) = self.balancing_state.get_mut(c) {
            *v += 1;
            *v - 1
        } else {
            self.balancing_state.insert(*c, 1);
            0
        }
    }

    fn pop_symbol(&mut self, c: &char) -> Result<BalancingDepthType, LexError<'a>> {
        if let Some(v) = self.balancing_state.get_mut(&Self::map_balance(&c)) {
            if *v >= 1 {
                *v -= 1;
                Ok(*v)
            } else {
                Err(LexError::MisbalancedSymbol {
                    symbol: *c,
                    open: Self::map_balance(&c),
                })
            }
        } else {
            Err(LexError::MisbalancedSymbol {
                symbol: *c,
                open: Self::map_balance(&c),
            })
        }
    }

    fn transform_to_type(&mut self, c: char) -> Result<TokenType<'a>, LexError<'a>> {
        match c {
            '(' | '[' | '{' => Ok(tok!(Punct c (Open self.push_symbol(&c)))),
            ')' | ']' | '}' => Ok(tok!(Punct c (Close self.pop_symbol(&c)?))),
            ';' => Ok(tok!(Punct c (Separator))),
            '"' | '\'' => self.parse_string(c),
            c => self.parse_identifier(c),
        }
    }

    fn parse_identifier(&mut self, first_char: char) -> Result<TokenType<'a>, LexError<'a>> {
        let start_offset = self.codepoint_offset - first_char.len_utf8();

        while let Some(c) = self.chars.peek() {
            if c.is_whitespace() {
                break;
            }

            self.consume_char();
        }

        Ok(TokenType::Identifier(
            &self.input[start_offset..self.codepoint_offset],
        ))
    }

    fn parse_string(&mut self, delimiter: char) -> Result<TokenType<'a>, LexError<'a>> {
        let mut start_offset = self.codepoint_offset;
        let mut is_escaped = false;
        let mut string = String::new();

        loop {
            if is_escaped {
                match self.consume_char() {
                    Some(c) => {
                        start_offset = self.codepoint_offset;
                        string.push(c);
                    }
                    None => {
                        return Err(LexError::UnexpectedEof {
                            expected: delimiter.to_string(),
                        })
                    }
                }

                is_escaped = false;
                continue;
            }

            match self.consume_char() {
                Some(c) if c == delimiter => break,
                Some('\\') => {
                    string.push_str(
                        &self.input[start_offset..(self.codepoint_offset - '\\'.len_utf8())],
                    );
                    is_escaped = true;
                }
                Some(_) => (),
                None => {
                    return Err(LexError::UnexpectedEof {
                        expected: delimiter.to_string(),
                    })
                }
            }
        }

        string.push_str(&self.input[start_offset..(self.codepoint_offset - '\\'.len_utf8())]);
        Ok(TokenType::String(string))
    }

    pub fn consume_char(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(c) => {
                self.codepoint_offset += c.len_utf8();
                if c == '\n' {
                    self.cur_line += 1;
                    self.cur_column = 1;
                } else {
                    self.cur_column += 1;
                }
                Some(c)
            }
            None => None,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.consume_char();
        }
    }

    fn skip_comments(&mut self) {
        if self.chars.peek() != Some(&'#') {
            return;
        }

        while let Some(c) = self.consume_char() {
            if c == '\n' {
                break;
            }
        }

        self.skip_whitespace();
    }

    pub fn next_token(&mut self) -> Result<TokenType<'a>, LexError<'a>> {
        self.skip_whitespace();
        self.skip_comments();

        if let Some(c) = self.consume_char() {
            self.transform_to_type(c)
        } else {
            Ok(TokenType::Eof)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tok;

    use super::*;

    fn tokens<'a>(input: &'a str) -> Result<Vec<TokenType<'a>>, LexError<'a>> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            match lexer.next_token() {
                Ok(TokenType::Eof) => break,
                Ok(token) => tokens.push(token),
                Err(err) => panic!("Unexpected error: {err:?}"),
            }
        }
        Ok(tokens)
    }

    #[test]
    fn it_works() {
        assert_eq!(
            tokens("if () { echo \"This is a \\\"test\\\".\" }"),
            Ok(vec![
                tok!(Id "if"),
                tok!(Punct '(' (Open 0)),
                tok!(Punct ')' (Close 0)),
                tok!(Punct '{' (Open 0)),
                tok!(Id "echo"),
                tok!(Str r#"This is a "test"."#),
                tok!(Punct '}' (Close 0)),
            ])
        );

        assert_eq!(
            tokens("(()())"),
            Ok(vec![
                tok!(Punct '(' (Open 0)),
                tok!(Punct '(' (Open 1)),
                tok!(Punct ')' (Close 1)),
                tok!(Punct '(' (Open 1)),
                tok!(Punct ')' (Close 1)),
                tok!(Punct ')' (Close 0)),
            ])
        );
    }
}
