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

#[cfg(test)]
mod tests {
    use crate::{FileDescriptor, RedirectMode};

    use super::*;

    #[test]
    fn default_command_is_empty() {
        let command = Command::default();
        assert_eq!(
            command,
            Command {
                arguments: vec![],
                redirects: vec![]
            }
        );
    }

    #[test]
    fn command_arguments_can_be_appended() {
        let arg = Word::Literal("argument".into());
        let mut command = Command::default();
        command.arg(arg.clone());
        assert_eq!(command.arguments, vec![arg])
    }

    #[test]
    fn command_redirects_can_be_appended() {
        let redirect = Redirect::new(
            FileDescriptor::Number(2),
            FileDescriptor::Number(1),
            RedirectMode::Write,
        );
        let mut command = Command::default();
        command.redirect(redirect.clone());
        assert_eq!(command.redirects, vec![redirect])
    }
}
