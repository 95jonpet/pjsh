use clap::Parser;
use pjsh_core::command::{Action, Args, Command, CommandResult};

use crate::{status, utils};

/// Command name.
const NAME: &str = "interpolate";

/// Interpolate text from the shell's environment.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct InterpolateOpts {
    /// Text to interpolate.
    #[clap(required = true, min_values = 1)]
    text: Vec<String>,
}

/// Implementation for the "interpolate" built-in command.
#[derive(Clone)]
pub struct Interpolate;
impl Command for Interpolate {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, mut args: Args) -> CommandResult {
        match InterpolateOpts::try_parse_from(args.context.lock().args()) {
            Ok(opts) => interpolate_text_args(opts),
            Err(error) => utils::exit_with_parse_error(&mut args.io, error),
        }
    }
}

/// Interpolates text arguments.
///
/// Writes one interpolated value per line to stdout.
/// Writes errors to stdout.
///
/// Interpolation is performed by the shell and additional file descriptors are
/// supplied by the executor. Thus, this function takes no arguments for I/O.
///
/// Returns 0 if all commands can be interpolated successfully, or 1 if at least
/// one argument cannot be interpolated.
fn interpolate_text_args(args: InterpolateOpts) -> CommandResult {
    let mut actions = Vec::with_capacity(args.text.len());

    for text in args.text {
        let action = Action::Interpolate(
            text,
            Box::new(|mut io, result| match result {
                Ok(interpolated) => {
                    let _ = writeln!(io.stdout, "{}", &interpolated);
                    status::SUCCESS
                }
                Err(error_message) => {
                    let _ = writeln!(io.stderr, "{}: {}", NAME, error_message);
                    status::GENERAL_ERROR
                }
            }),
        );
        actions.push(action);
    }

    CommandResult::with_actions(status::SUCCESS, actions)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    use parking_lot::Mutex;
    use pjsh_core::{Context, Scope};

    use crate::utils::empty_io;

    use super::*;

    #[test]
    fn it_interpolate_input() {
        let interpolate = Interpolate {};
        let ctx = Arc::new(Mutex::new(Context::with_scopes(vec![Scope::new(
            "scope".into(),
            Some(vec!["interpolate".into(), "word".into()]),
            Some(HashMap::default()),
            None,
            HashSet::default(),
            false,
        )])));
        let args = Args::new(ctx, empty_io());

        let result = interpolate.run(args);

        assert_eq!(result.code, status::SUCCESS);
        assert_eq!(result.actions.len(), 1);
        if let Action::Interpolate(arg, _) = &result.actions[0] {
            assert_eq!(arg, "word");
        } else {
            unreachable!()
        }
    }
}
