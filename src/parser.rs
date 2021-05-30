use crate::shell::Shell;
use crate::token::Operator;
use crate::token::Token;

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
            _ => self.get_stdout(),
        }
    }

    pub fn get_stdout(&mut self) -> Option<Stdio> {
        match self {
            FileDescriptor::Stdin => Some(Stdio::from(dup_stdin().unwrap())),
            FileDescriptor::Stdout => Some(Stdio::from(dup_stdout().unwrap())),
            FileDescriptor::Stderr => Some(Stdio::from(dup_stderr().unwrap())),
            FileDescriptor::PipeOut(writer) => Some(Stdio::from(writer.try_clone().unwrap())),
            FileDescriptor::PipeIn(reader) => Some(Stdio::from(reader.try_clone().unwrap())),
        }
    }

    pub fn get_stderr(&mut self) -> Option<Stdio> {
        self.get_stdout()
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
    #[allow(dead_code)]
    shell: Rc<RefCell<Shell>>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(lexer: I, shell: Rc<RefCell<Shell>>) -> Self {
        Self {
            lexer: lexer.peekable(),
            shell,
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
        if let Some(Token::Operator(Operator::Bang)) = self.lexer.peek() {
            self.lexer.next();
            return Ok(Cmd::Not(Box::new(self.get_single()?)));
        }

        let mut result: Vec<String> = Vec::new();
        let io = Io::new();

        loop {
            match self.lexer.peek() {
                Some(Token::Word(_)) => {
                    if let Some(Token::Word(word)) = self.lexer.next() {
                        result.push(word);
                    } else {
                        unreachable!()
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
