use crate::token::{Literal, Separator, Token};

use os_pipe::{dup_stderr, dup_stdin, dup_stdout, PipeReader, PipeWriter};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::{Iterator, Peekable};
use std::process::{Command, Output, Stdio};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum FileDescriptor {
    Stdin,
    Stdout,
    Stderr,
}

impl FileDescriptor {
    pub fn get_stdin(self) -> Option<Stdio> {
        match self {
            _ => dup_stdin().map(|io| Stdio::from(io)).ok(),
        }
    }

    pub fn get_stdout(self) -> Option<Stdio> {
        match self {
            _ => dup_stdout().map(|io| Stdio::from(io)).ok(),
        }
    }

    pub fn get_stderr(self) -> Option<Stdio> {
        match self {
            _ => dup_stderr().map(|io| Stdio::from(io)).ok(),
        }
    }
}

pub struct Io {
    stdin: Rc<RefCell<FileDescriptor>>,
    stdout: Rc<RefCell<FileDescriptor>>,
    stderr: Rc<RefCell<FileDescriptor>>,
}

impl Io {
    fn new() -> Io {
        Io {
            stdin: Rc::new(RefCell::new(FileDescriptor::Stdin)),
            stdout: Rc::new(RefCell::new(FileDescriptor::Stdout)),
            stderr: Rc::new(RefCell::new(FileDescriptor::Stderr)),
        }
    }

    fn set_stdin(&mut self, fd: Rc<RefCell<FileDescriptor>>) {
        self.stdin = fd;
    }

    fn set_stdout(&mut self, fd: Rc<RefCell<FileDescriptor>>) {
        self.stdout = fd;
    }

    fn set_stderr(&mut self, fd: Rc<RefCell<FileDescriptor>>) {
        self.stderr = fd;
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
    fn new(cmd: String, args: Vec<String>, io: Io) -> Self {
        Self {
            cmd,
            args,
            env: None,
            stdin: io.stdin,
            stdout: io.stdout,
            stderr: io.stderr,
        }
    }

    // pub fn execute(self) -> std::io::Result<Output> {
    //     let mut command = Command::new(self.cmd);
    //     command.args(&self.args);

    //     if let Some(stdout) = self.stdout.borrow_mut().get_stdout() {
    //         command.stdout(stdout);
    //     }

    //     // if let Some(stdin) = self.stdin.as_ref().borrow().get_stdin() {
    //     //     command.stdin(Stdio::from(stdin));
    //     // }

    //     // if let Some(stdout) = self.stdout.as_ref().borrow().get_stdout() {
    //     //     command.stdout(Stdio::from(stdout));
    //     // }

    //     // if let Some(stderr) = self.stderr.as_ref().borrow().get_stderr() {
    //     //     command.stderr(Stdio::from(stderr));
    //     // }

    //     command.output()
    // }
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
        let node = self.get_pipe()?;
        Ok(node)
    }

    fn get_pipe(&mut self) -> Result<Cmd, String> {
        let node = self.get_single()?;
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

        let cmd = SingleCommand::new(result.remove(0), result, Io::new());
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
