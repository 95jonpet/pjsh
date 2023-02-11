use clap::Parser;
use pjsh_core::{
    command::{Action, Args, Command, CommandResult},
    utils::path_to_string,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "which";

/// Print the full path of commands to standard output.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct WhichOpts {
    /// Command names to resolve.
    #[clap(required = true, num_args = 1..)]
    name: Vec<String>,
}

/// Implementation for the "which" built-in command.
#[derive(Clone)]
pub struct Which;
impl Command for Which {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, args: &mut Args) -> CommandResult {
        match WhichOpts::try_parse_from(args.context.args()) {
            Ok(opts) => resolve_command_paths(opts),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Resolves command paths.
///
/// Writes one resolved path per line to stdout.
/// Writes errors to stdout.
///
/// Command resolution is performed by the shell and additional file descriptors
/// are supplied by the executor. Thus, this function takes no arguments for
/// handling I/O.
///
/// Returns 0 if all commands can be resolved successfully, or 1 if at least
/// one argument cannot be resolved.
fn resolve_command_paths(args: WhichOpts) -> CommandResult {
    let mut actions = Vec::with_capacity(args.name.len());

    for name in args.name {
        let action = Action::ResolveCommandPath(
            name.clone(),
            Box::new(|name, mut io, path| {
                if let Some(path) = path {
                    let _ = writeln!(io.stdout, "{}", path_to_string(path));
                    status::SUCCESS
                } else {
                    let _ = writeln!(io.stderr, "{NAME}: no '{name}' in path.");
                    status::GENERAL_ERROR
                }
            }),
        );
        actions.push(action);
    }

    CommandResult::with_actions(status::SUCCESS, actions)
}
