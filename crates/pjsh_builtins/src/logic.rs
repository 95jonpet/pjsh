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

    fn run(&self, mut args: Args) -> CommandResult {
        match TrueOpts::try_parse_from(args.context.lock().args()) {
            Ok(_) => CommandResult::code(status::SUCCESS),
            Err(error) => exit_with_parse_error(&mut args.io, error),
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

    fn run(&self, mut args: Args) -> CommandResult {
        match TrueOpts::try_parse_from(args.context.lock().args()) {
            Ok(_) => CommandResult::code(1), // Any non-zero code is false.
            Err(error) => exit_with_parse_error(&mut args.io, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;
    use pjsh_core::Context;

    use crate::utils::empty_io;

    use super::*;

    #[test]
    fn true_exits_with_zero_code() {
        let ctx = Context::default();
        let command = True {};

        let args = Args::new(Arc::new(Mutex::new(ctx)), empty_io());
        let result = command.run(args);

        assert_eq!(result.code, 0);
        assert!(result.actions.is_empty());
    }

    #[test]
    fn false_exits_with_non_zero_code() {
        let ctx = Context::default();
        let command = False {};

        let args = Args::new(Arc::new(Mutex::new(ctx)), empty_io());
        let result = command.run(args);

        assert_ne!(result.code, 0);
        assert!(result.actions.is_empty());
    }
}
