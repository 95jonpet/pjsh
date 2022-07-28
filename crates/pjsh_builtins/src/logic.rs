use clap::Parser;
use pjsh_core::{
    command::Io,
    command::{Args, Command, CommandResult},
};

use crate::status;

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
        if let Err(error) = TrueOpts::try_parse_from(args.context.lock().args()) {
            print_error(&mut args.io, error);
        };

        CommandResult::code(status::SUCCESS)
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
        if let Err(error) = TrueOpts::try_parse_from(args.context.lock().args()) {
            print_error(&mut args.io, error);
        };

        CommandResult::code(1) // Any non-zero code is false.
    }
}

/// Prints a [`clap::Error`] to a relevant file descriptor.
fn print_error(io: &mut Io, error: clap::Error) {
    let _ = match error.use_stderr() {
        true => writeln!(io.stderr, "{}", error),
        false => writeln!(io.stdout, "{}", error),
    };
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
