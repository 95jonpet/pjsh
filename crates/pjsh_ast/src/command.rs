use crate::{Redirect, Word};

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub program: Word,
    pub arguments: Vec<Word>,
    pub redirects: Vec<Redirect>,
}

impl Command {
    /// Constructs a new command for calling a program.
    pub fn new(program: Word) -> Self {
        Self {
            program,
            arguments: Vec::new(),
            redirects: Vec::new(),
        }
    }

    /// Appends an argument to the command.
    pub fn arg(&mut self, arg: Word) -> &mut Self {
        self.arguments.push(arg);
        self
    }

    /// Appends a redirect to the command's redirection list.
    pub fn redirect(&mut self, redirect: Redirect) -> &mut Self {
        self.redirects.push(redirect);
        self
    }
}
