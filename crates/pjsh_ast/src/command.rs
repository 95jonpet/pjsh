use crate::{Redirect, Word};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Command {
    pub arguments: Vec<Word>,
    pub redirects: Vec<Redirect>,
}

impl Command {
    /// Appends an argument to the command.
    pub fn arg(&mut self, arg: Word) {
        self.arguments.push(arg);
    }

    /// Appends a redirect to the command's redirection list.
    pub fn redirect(&mut self, redirect: Redirect) {
        self.redirects.push(redirect);
    }
}
