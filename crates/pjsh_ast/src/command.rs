use crate::{Redirect, Word};

#[derive(Debug, PartialEq, Eq)]
pub struct Command<'a> {
    pub program: Word<'a>,
    pub arguments: Vec<Word<'a>>,
    pub redirects: Vec<Redirect>,
}

impl<'a> Command<'a> {
    /// Constructs a new command for calling a program.
    pub fn new(program: Word<'a>) -> Self {
        Self {
            program,
            arguments: Vec::new(),
            redirects: Vec::new(),
        }
    }

    /// Appends an argument to the command.
    pub fn arg<'b>(&'b mut self, arg: Word<'a>) -> &'b mut Self {
        self.arguments.push(arg);
        self
    }

    /// Appends a redirect to the command's redirection list.
    pub fn redirect<'b>(&'b mut self, redirect: Redirect) -> &'b mut Self {
        self.redirects.push(redirect);
        self
    }
}
