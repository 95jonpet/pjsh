use clap::Parser;
use pjsh_core::{
    command::Io,
    command::{Action, Args, Command, CommandResult, CommandType},
    utils::path_to_string,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "type";

/// Display information about command types.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct TypeOpts {
    /// Command names to resolve.
    #[clap(required = true, num_args = 1..)]
    name: Vec<String>,
}

/// Implementation for the "type" built-in command.
#[derive(Clone)]
pub struct Type;
impl Command for Type {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, args: &mut Args) -> CommandResult {
        match TypeOpts::try_parse_from(args.context.args()) {
            Ok(opts) => resolve_command_types(opts),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Resolves command types.
///
/// Writes one resolved type per line to stdout.
/// Writes errors to stdout.
///
/// Command resolution is performed by the shell and additional file descriptors
/// are supplied by the executor. Thus, this function takes no arguments for
/// handling I/O.
///
/// Returns 0 if all commands can be resolved successfully, or 1 if at least
/// one argument cannot be resolved.
fn resolve_command_types(args: TypeOpts) -> CommandResult {
    let mut actions = Vec::with_capacity(args.name.len());

    for name in args.name {
        let action = Action::ResolveCommandType(
            name.clone(),
            Box::new(|io, name, r#type| print_type(name, r#type, io)),
        );
        actions.push(action);
    }

    CommandResult::with_actions(status::SUCCESS, actions)
}

/// Prints the type of a command to stdout.
///
/// Returns an exit code.
fn print_type(name: String, r#type: CommandType, mut io: Io) -> i32 {
    match r#type {
        CommandType::Alias(alias) => {
            let _ = writeln!(io.stdout, "{name} is aliased to '{alias}'");
            status::SUCCESS
        }
        CommandType::Builtin => {
            let _ = writeln!(io.stdout, "{name} is a shell built-in");
            status::SUCCESS
        }
        CommandType::Function => {
            let _ = writeln!(io.stdout, "{name} is a function");
            status::SUCCESS
        }
        CommandType::Program(path) => {
            let _ = writeln!(io.stdout, "{name} is '{}'", path_to_string(path));
            status::SUCCESS
        }
        CommandType::Unknown => {
            let _ = writeln!(io.stderr, "{NAME}: {name}: not found");
            status::GENERAL_ERROR
        }
    }
}
