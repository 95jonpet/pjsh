use pjsh_core::command::{Args, Command, CommandResult};

use crate::status;

/// Implementation for the "true" built-in command.
///
/// Exits with a code status indicating success.
#[derive(Clone)]
pub struct True;
impl Command for True {
    fn name(&self) -> &str {
        "true"
    }

    fn run<'a>(&self, _args: &'a mut Args) -> CommandResult {
        CommandResult::code(status::SUCCESS)
    }
}

/// Implementation for the "false" built-in command.
///
/// Exits with a status code indicating failure.
#[derive(Clone)]
pub struct False;
impl Command for False {
    fn name(&self) -> &str {
        "false"
    }

    fn run<'a>(&self, _args: &'a mut Args) -> CommandResult {
        CommandResult::code(1) // Any non-zero code is false.
    }
}

#[cfg(test)]
mod tests {
    use pjsh_core::Context;

    use crate::utils::empty_io;

    use super::*;

    #[test]
    fn true_exits_with_zero_code() {
        let mut ctx = Context::default();
        let mut io = empty_io();
        let command = True {};

        let mut args = Args::new(&mut ctx, &mut io);
        if let CommandResult::Builtin(result) = command.run(&mut args) {
            assert_eq!(result.code, 0);
            assert!(result.actions.is_empty());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn false_exits_with_non_zero_code() {
        let mut ctx = Context::default();
        let mut io = empty_io();
        let command = False {};

        let mut args = Args::new(&mut ctx, &mut io);
        if let CommandResult::Builtin(result) = command.run(&mut args) {
            assert_ne!(result.code, 0);
            assert!(result.actions.is_empty());
        } else {
            unreachable!()
        }
    }
}
