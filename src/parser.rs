use crate::token::{Literal, Separator, Token};

use std::iter::{Iterator, Peekable};
use std::process::Command;

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    lexer: Peekable<I>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(lexer: I) -> Self {
        Self {
            lexer: lexer.peekable(),
        }
    }

    pub fn parse(self) -> Vec<Command> {
        let mut groups: Vec<Vec<String>> = Vec::new();
        let mut group: Vec<String> = Vec::new();
        for token in self.lexer {
            match token {
                Token::Separator(Separator::Semicolon) => {
                    groups.push(group);
                    group = Vec::new();
                }
                _ => {
                    if let Some(string) = Self::token_to_string(token) {
                        group.push(string);
                    }
                }
            }
        }

        if !groups.contains(&group) {
            groups.push(group);
        }

        let mut commands: Vec<Command> = Vec::new();
        for group in groups {
            if group.is_empty() {
                break;
            }

            let mut command = Command::new(&group[0]);
            command.args(&group[1..]);
            commands.push(command);
        }

        commands
    }

    fn token_to_string(token: Token) -> Option<String> {
        match token {
            Token::Identifier(id) => Some(id),
            Token::Literal(Literal::String(string)) => Some(string),
            Token::Literal(Literal::Integer(integer)) => Some(integer.to_string()),
            _ => None,
        }
    }
}
