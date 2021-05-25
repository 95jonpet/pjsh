use crate::token::Operator;
use crate::token::{Literal, Token};

use os_pipe::{dup_stderr, dup_stdin, dup_stdout, PipeReader, PipeWriter};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::{Iterator, Peekable};
use std::process::Stdio;
use std::rc::Rc;

#[derive(Debug)]
pub enum FileDescriptor {
    Stdin,
    Stdout,
    Stderr,
    PipeOut(PipeWriter),
    PipeIn(PipeReader),
}

impl PartialEq for FileDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.variant() == other.variant()
    }
}

impl FileDescriptor {
    fn variant(&self) -> &str {
        match *self {
            FileDescriptor::Stdin => "Stdin",
            FileDescriptor::Stdout => "Stdout",
            FileDescriptor::Stderr => "Stderr",
            FileDescriptor::PipeOut(_) => "PipeOut",
            FileDescriptor::PipeIn(_) => "PipeIn",
        }
    }

    pub fn get_stdin(&mut self) -> Option<Stdio> {
        match self {
            _ => dup_stdin().map(|io| Stdio::from(io)).ok(),
        }
    }

    pub fn get_stdout(&mut self) -> Option<Stdio> {
        match self {
            _ => dup_stdout().map(|io| Stdio::from(io)).ok(),
        }
    }

    pub fn get_stderr(&mut self) -> Option<Stdio> {
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
}

#[derive(Debug)]
pub enum Cmd {
    Single(SingleCommand),
    Pipeline(Box<Cmd>, Box<Cmd>),
    And(Box<Cmd>, Box<Cmd>),
    Or(Box<Cmd>, Box<Cmd>),
    Not(Box<Cmd>),
    Empty,
}

#[derive(Debug)]
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
        self.get_and_or_or()
    }

    fn get_and_or_or(&mut self) -> Result<Cmd, String> {
        let mut node = self.get_pipe()?;
        while let Some(Token::Operator(Operator::And)) | Some(Token::Operator(Operator::Or)) =
            self.lexer.peek()
        {
            match self.lexer.next() {
                Some(Token::Operator(Operator::And)) => {
                    node = Cmd::And(Box::new(node), Box::new(self.get_pipe()?));
                }
                Some(Token::Operator(Operator::Or)) => {
                    node = Cmd::Or(Box::new(node), Box::new(self.get_pipe()?));
                }
                _ => unreachable!(),
            };
        }
        Ok(node)
    }

    fn get_pipe(&mut self) -> Result<Cmd, String> {
        let mut node = self.get_single()?;
        while let Some(Token::Operator(Operator::Pipe)) = self.lexer.peek() {
            self.lexer.next();
            node = Cmd::Pipeline(Box::new(node), Box::new(self.get_single()?));
        }
        Ok(node)
    }

    fn get_single(&mut self) -> Result<Cmd, String> {
        let mut result: Vec<String> = Vec::new();
        let io = Io::new();

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
                Some(Token::Literal(Literal::Integer(_))) => {
                    if let Some(Token::Literal(Literal::Integer(int))) = self.lexer.next() {
                        result.push(int.to_string());
                    }
                }
                _ => break,
            }
        }

        if result.is_empty() {
            unimplemented!("Missing result");
        }

        let cmd = SingleCommand::new(result.remove(0), result, io);
        Ok(Cmd::Single(cmd))
    }
}
