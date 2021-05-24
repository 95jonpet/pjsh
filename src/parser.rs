use crate::token::{Literal, Separator, Token};

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Stderr;
use std::io::Stdin;
use std::io::Stdout;
use std::iter::{Iterator, Peekable};
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum FileDescriptor {
    Stdin,
    Stdout,
    Stderr,
}

impl FileDescriptor {
    pub fn get_stdin(self) -> Stdin {
        std::io::stdin()
    }

    pub fn get_stdout(self) -> Stdout {
        std::io::stdout()
    }
    pub fn get_stderr(self) -> Stderr {
        std::io::stderr()
    }
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Single(SingleCommand),
    Pipeline(Box<Cmd>, Box<Cmd>),
    And(Box<Cmd>, Box<Cmd>),
    Or(Box<Cmd>, Box<Cmd>),
    Not(Box<Cmd>),
    Empty,
}

#[derive(Debug, PartialEq)]
pub struct SingleCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    pub stdin: Rc<RefCell<FileDescriptor>>,
    pub stdout: Rc<RefCell<FileDescriptor>>,
    pub stderr: Rc<RefCell<FileDescriptor>>,
}

impl SingleCommand {
    fn new(
        cmd: String,
        args: Vec<String>,
        stdin: Rc<RefCell<FileDescriptor>>,
        stdout: Rc<RefCell<FileDescriptor>>,
        stderr: Rc<RefCell<FileDescriptor>>,
    ) -> Self {
        Self {
            cmd,
            args,
            env: None,
            stdin,
            stdout,
            stderr,
        }
    }

    pub fn execute(self) {
        let mut command = Command::new(self.cmd);
        command.args(self.args);
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.output();
    }
}

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

    pub fn get(&mut self) -> Result<Cmd, String> {
        self.get_and()
    }

    fn get_and(&mut self) -> Result<Cmd, String> {
        let mut node = self.get_pipe()?;
        Ok(node)
    }

    fn get_pipe(&mut self) -> Result<Cmd, String> {
        let mut node = self.get_single()?;
        Ok(node)
    }

    fn get_single(&mut self) -> Result<Cmd, String> {
        let mut result: Vec<String> = Vec::new();
        loop {
            match self.lexer.peek() {
                Some(Token::Identifier(_)) => {
                    if let Some(Token::Identifier(id)) = self.lexer.next() {
                        result.push(id);
                    }
                }
                Some(Token::Literal(Literal::String(_))) => {
                    if let Some(Token::Literal(Literal::String(string))) = self.lexer.next() {
                        result.push(string);
                    }
                }
                _ => break,
            }
        }

        if result.is_empty() {
            unimplemented!("Missing result");
        }

        let cmd = SingleCommand::new(
            result.remove(0),
            result,
            Rc::new(RefCell::new(FileDescriptor::Stdin)),
            Rc::new(RefCell::new(FileDescriptor::Stdout)),
            Rc::new(RefCell::new(FileDescriptor::Stderr)),
        );
        Ok(Cmd::Single(cmd))
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
