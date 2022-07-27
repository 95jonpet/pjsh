use clap::Parser;
use pjsh_core::command::{Args, Command, CommandResult};

use crate::utils;

/// Exit the shell.
///
/// If no exit status is supplied, the last command's exit code is used.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = "exit", version)]
struct ExitOpts {
    /// Exit status for the shell.
    status: Option<i32>,
}

/// Implementation for the "exit" built-in command.
#[derive(Clone)]
pub struct Exit;
impl Command for Exit {
    fn name(&self) -> &str {
        "exit"
    }

    fn run(&self, mut args: Args) -> CommandResult {
        let ctx = args.context.lock();
        match ExitOpts::try_parse_from(ctx.args()) {
            Ok(opts) => match opts.status {
                Some(status) => CommandResult::code(status),
                None => CommandResult::code(ctx.last_exit()),
            },
            Err(error) => utils::exit_with_parse_error(&mut args.io, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use pjsh_core::{Context, Scope};

    use crate::utils::empty_io;

    use super::*;

    #[test]
    fn it_uses_the_last_exit_code_by_default() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["exit".to_owned()],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        ctx.register_exit(17);
        let exit = Exit {};

        let args = Args::from_context(ctx, empty_io());
        let result = exit.run(args);

        assert_eq!(result.code, 17);
        assert!(result.actions.is_empty());
    }

    #[test]
    fn it_can_use_code_from_argument() {
        let ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["exit".to_owned(), "1".to_owned()],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        let exit = Exit {};

        let args = Args::from_context(ctx, empty_io());
        let result = exit.run(args);

        assert_eq!(result.code, 1);
        assert!(result.actions.is_empty());
    }

    #[test]
    fn it_exits_with_code_2_if_code_argument_is_invalid() {
        let ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["exit".to_owned(), "non-integer".to_owned()],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        let exit = Exit {};

        let args = Args::from_context(ctx, empty_io());
        let result = exit.run(args);

        assert_eq!(result.code, 2); // Exit 2 = misuse of shell built-in.
        assert!(result.actions.is_empty());
    }
}
