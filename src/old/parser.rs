use crate::old::shell::Shell;
use crate::old::token::Token;
use os_pipe::{dup_stderr, dup_stdin, dup_stdout, PipeReader, PipeWriter};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::{Iterator, Peekable};
use std::process::Stdio;
use std::rc::Rc;

use super::token::{Operator, Unit};

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

    /// Returns the [`Stdio`] instance mapped to stdin.
    pub fn get_stdin(&mut self) -> Option<Stdio> {
        match self {
            _ => self.get_stdout(),
        }
    }

    /// Returns the [`Stdio`] instance mapped to stdout.
    pub fn get_stdout(&mut self) -> Option<Stdio> {
        match self {
            FileDescriptor::Stdin => Some(Stdio::from(dup_stdin().unwrap())),
            FileDescriptor::Stdout => Some(Stdio::from(dup_stdout().unwrap())),
            FileDescriptor::Stderr => Some(Stdio::from(dup_stderr().unwrap())),
            FileDescriptor::PipeOut(writer) => Some(Stdio::from(writer.try_clone().unwrap())),
            FileDescriptor::PipeIn(reader) => Some(Stdio::from(reader.try_clone().unwrap())),
        }
    }

    /// Returns the [`Stdio`] instance mapped to stderr.
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
    pub fn new() -> Io {
        Io {
            stdin: Rc::new(RefCell::new(FileDescriptor::Stdin)),
            stdout: Rc::new(RefCell::new(FileDescriptor::Stdout)),
            stderr: Rc::new(RefCell::new(FileDescriptor::Stderr)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Simple(SimpleCommand),
    Pipeline(Box<Cmd>, Box<Cmd>),
    And(Box<Cmd>, Box<Cmd>),
    Or(Box<Cmd>, Box<Cmd>),
    Not(Box<Cmd>),
    NoOp,
}

#[derive(Debug, PartialEq)]
pub struct SimpleCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub stdin: Rc<RefCell<FileDescriptor>>,
    pub stdout: Rc<RefCell<FileDescriptor>>,
    pub stderr: Rc<RefCell<FileDescriptor>>,
}

impl SimpleCommand {
    pub fn new(cmd: String, args: Vec<String>, io: Io, env: HashMap<String, String>) -> Self {
        Self {
            cmd,
            args,
            env: env,
            stdin: io.stdin,
            stdout: io.stdout,
            stderr: io.stderr,
        }
    }
}

struct ParserOptions {
    allow_unresolved_variables: bool,
}

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    lexer: Peekable<I>,
    #[allow(dead_code)]
    shell: Rc<RefCell<Shell>>,
    options: ParserOptions,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(lexer: I, shell: Rc<RefCell<Shell>>) -> Self {
        let options = ParserOptions {
            allow_unresolved_variables: true,
        };

        Self {
            lexer: lexer.peekable(),
            shell,
            options,
        }
    }

    pub fn get(&mut self) -> Result<Cmd, String> {
        self.get_and_or_or()
    }

    fn get_and_or_or(&mut self) -> Result<Cmd, String> {
        let mut node = self.get_pipeline()?;
        while let Some(Token::Operator(Operator::And)) | Some(Token::Operator(Operator::Or)) =
            self.lexer.peek()
        {
            match self.lexer.next() {
                Some(Token::Operator(Operator::And)) => {
                    node = Cmd::And(Box::new(node), Box::new(self.get_pipeline()?));
                }
                Some(Token::Operator(Operator::Or)) => {
                    node = Cmd::Or(Box::new(node), Box::new(self.get_pipeline()?));
                }
                _ => unreachable!(),
            };
        }
        Ok(node)
    }

    fn get_pipeline(&mut self) -> Result<Cmd, String> {
        let mut node = self.get_simple()?;
        while let Some(Token::Operator(Operator::Pipe)) = self.lexer.peek() {
            self.lexer.next();
            node = Cmd::Pipeline(Box::new(node), Box::new(self.get_simple()?));
        }
        Ok(node)
    }

    fn get_simple(&mut self) -> Result<Cmd, String> {
        if let Some(Token::Operator(Operator::Bang)) = self.lexer.peek() {
            self.lexer.next();
            return Ok(Cmd::Not(Box::new(self.get_simple()?)));
        }

        let mut env: HashMap<String, String> = HashMap::new();
        let mut result: Vec<String> = Vec::new();
        let io = Io::new();

        loop {
            match self.lexer.peek() {
                Some(Token::Word(_)) => {
                    if let Some(Token::Word(units)) = self.lexer.next() {
                        let expanded_word = self.expand_word(&units);
                        result.push(expanded_word);
                    } else {
                        unreachable!()
                    }
                }
                Some(Token::Assign(key, value)) => {
                    env.insert(key.to_owned(), value.to_owned());
                    self.lexer.next();
                }
                Some(Token::Comment(_)) => {
                    self.lexer.next();
                }
                Some(Token::Separator(crate::old::token::Separator::Semicolon)) => break,
                _ => break,
            }
        }

        if result.is_empty() {
            if !env.is_empty() {
                self.shell.borrow_mut().vars.extend(env);
            }

            return Ok(Cmd::NoOp);
        }

        Ok(Cmd::Simple(SimpleCommand::new(
            result.remove(0),
            result,
            io,
            env,
        )))
    }

    fn expand_word(&self, units: &Vec<Unit>) -> String {
        let mut word = String::new();
        for unit in units {
            match unit {
                Unit::Literal(literal) => word.push_str(literal),
                Unit::Variable(var) => match &self.shell.borrow_mut().get_var(var) {
                    Some(value) => word.push_str(value),
                    None => {
                        if !self.options.allow_unresolved_variables {
                            panic!("Unresolved variable '{}'", var)
                        }
                    }
                },
            }
        }
        word
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn it_parses_single_commands() {
        let tokens = vec![
            Token::Word(vec![Unit::Literal(String::from("ls"))]),
            Token::Word(vec![Unit::Literal(String::from("-lah"))]),
        ];

        let expected_ast = Cmd::Simple(SimpleCommand::new(
            String::from("ls"),
            vec![String::from("-lah")],
            Io::new(),
            HashMap::new(),
        ));

        assert_eq!(ast(tokens), Ok(expected_ast));
    }

    #[test]
    fn it_parses_pipelines() {
        let tokens = vec![
            Token::Word(vec![Unit::Literal(String::from("cat"))]),
            Token::Word(vec![Unit::Literal(String::from("my_file"))]),
            Token::Operator(Operator::Pipe),
            Token::Word(vec![Unit::Literal(String::from("grep"))]),
            Token::Word(vec![Unit::Literal(String::from("test"))]),
        ];

        let expected_ast = Cmd::Pipeline(
            Box::new(Cmd::Simple(SimpleCommand::new(
                String::from("cat"),
                vec![String::from("my_file")],
                Io::new(),
                HashMap::new(),
            ))),
            Box::new(Cmd::Simple(SimpleCommand::new(
                String::from("grep"),
                vec![String::from("test")],
                Io::new(),
                HashMap::new(),
            ))),
        );

        assert_eq!(ast(tokens), Ok(expected_ast));
    }

    #[test]
    fn it_parses_commands_with_and_logic() {
        let tokens = vec![
            Token::Word(vec![Unit::Literal(String::from("true"))]),
            Token::Operator(Operator::And),
            Token::Word(vec![Unit::Literal(String::from("false"))]),
        ];

        let expected_ast = Cmd::And(
            Box::new(Cmd::Simple(SimpleCommand::new(
                String::from("true"),
                vec![],
                Io::new(),
                HashMap::new(),
            ))),
            Box::new(Cmd::Simple(SimpleCommand::new(
                String::from("false"),
                vec![],
                Io::new(),
                HashMap::new(),
            ))),
        );

        assert_eq!(ast(tokens), Ok(expected_ast));
    }

    #[test]
    fn it_parses_commands_with_or_logic() {
        let tokens = vec![
            Token::Word(vec![Unit::Literal(String::from("false"))]),
            Token::Operator(Operator::Or),
            Token::Word(vec![Unit::Literal(String::from("true"))]),
        ];

        let expected_ast = Cmd::Or(
            Box::new(Cmd::Simple(SimpleCommand::new(
                String::from("false"),
                vec![],
                Io::new(),
                HashMap::new(),
            ))),
            Box::new(Cmd::Simple(SimpleCommand::new(
                String::from("true"),
                vec![],
                Io::new(),
                HashMap::new(),
            ))),
        );

        assert_eq!(ast(tokens), Ok(expected_ast));
    }

    /// Tests that assignments are put in command environment.
    #[test]
    fn it_assigns_variables() {
        let tokens = vec![
            Token::Assign(String::from("key"), String::from("value")),
            Token::Word(vec![Unit::Literal(String::from("echo"))]),
            Token::Word(vec![Unit::Literal(String::from("test"))]),
        ];

        let mut expected_env = HashMap::new();
        expected_env.insert(String::from("key"), String::from("value"));
        let expected_ast = Cmd::Simple(SimpleCommand::new(
            String::from("echo"),
            vec![String::from("test")],
            Io::new(),
            expected_env,
        ));

        assert_eq!(ast(tokens), Ok(expected_ast));
    }

    /// Parses a token iterator and returns an abstract syntax tree.
    fn ast(tokens: Vec<Token>) -> Result<Cmd, String> {
        let shell = Rc::new(RefCell::new(Shell::from_command(String::new())));
        let mut parser = Parser::new(tokens.into_iter(), shell);
        parser.get()
    }
}
