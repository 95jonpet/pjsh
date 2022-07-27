use crate::{Redirect, Word};

/// A command represents an action that should be executed within the shell.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Command {
    /// List of arguments for the command. The first argument represents the
    /// name of the program to execute.
    pub arguments: Vec<Word>,

    /// Input/output redirects to consider when executing the specific command.
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
