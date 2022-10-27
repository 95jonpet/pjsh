use clap::Parser;
use pjsh_core::command::{Args, Command, CommandResult};

use crate::{status, utils::exit_with_parse_error};

/// Exit with a code status indicating success.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = "true", version)]
struct TrueOpts;

/// Implementation for the "true" built-in command.
#[derive(Clone)]
pub struct True;
impl Command for True {
    fn name(&self) -> &str {
        "true"
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match TrueOpts::try_parse_from(args.context.args()) {
            Ok(_) => CommandResult::code(status::SUCCESS),
            Err(error) => exit_with_parse_error(args.io, error),
        }
    }
}

/// Exit with a status code indicating failure.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = "false", version)]
struct FalseOpts;

/// Implementation for the "false" built-in command.
#[derive(Clone)]
pub struct False;
impl Command for False {
    fn name(&self) -> &str {
        "false"
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match TrueOpts::try_parse_from(args.context.args()) {
            Ok(_) => CommandResult::code(1), // Any non-zero code is false.
            Err(error) => exit_with_parse_error(args.io, error),
        }
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
