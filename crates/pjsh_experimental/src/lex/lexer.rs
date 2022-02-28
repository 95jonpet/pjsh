use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
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

#[derive(Debug)]
pub enum TokenType<'a> {
    Eof,
    Punctuation { raw: char, kind: PunctuationKind },
    Operator(&'a str),
    Identifier(&'a str),
    String(&'a str),
}

#[derive(Debug)]
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
            '(' | '[' | '{' => Ok(TokenType::Punctuation {
                raw: c,
                kind: PunctuationKind::Open(self.push_symbol(&c)),
            }),
            ')' | ']' | '}' => Ok(TokenType::Punctuation {
                raw: c,
                kind: PunctuationKind::Close(self.pop_symbol(&c)?),
            }),
            '"' => self.parse_string('"'),
            _ => Err(LexError::UnknownSymbol {
                symbol: c.to_string(),
            }),
        }
    }

    fn parse_string(&mut self, delimiter: char) -> Result<TokenType<'a>, LexError<'a>> {
        let start_offset = self.codepoint_offset;
        let mut is_escaped = false;

        loop {
            if is_escaped {
                if self.consume_char().is_none() {
                    return Err(LexError::UnexpectedEof {
                        expected: delimiter.to_string(),
                    });
                }

                is_escaped = false;
                continue;
            }

            match self.consume_char() {
                Some(c) if c == delimiter => break,
                Some('\\') => is_escaped = true,
                Some(_) => (),
                None => {
                    return Err(LexError::UnexpectedEof {
                        expected: delimiter.to_string(),
                    })
                }
            }
        }

        let end_offset = self.codepoint_offset - delimiter.len_utf8();
        Ok(TokenType::String(&self.input[start_offset..end_offset]))
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

    pub fn next_token(&mut self) -> Result<TokenType<'a>, LexError<'a>> {
        self.skip_whitespace();

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

    fn tokens<'a>(input: &'a str) -> Vec<TokenType<'a>> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            match lexer.next_token() {
                Ok(TokenType::Eof) => break,
                Ok(token) => {
                    println!("{token:?}");
                    tokens.push(token)
                }
                Err(err) => eprintln!("{err:?}"),
            }
        }
        tokens
    }

    #[test]
    fn it_works() {
        println!("{:?}", tokens("if () { echo \"This is a \\\"test\\\".\" }"));
        // println!("{:?}", tokens("(()())"));

        // println!("{:?}", tok!(Eof));
    }
}
