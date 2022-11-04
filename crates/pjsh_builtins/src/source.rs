use std::path::PathBuf;

use clap::Parser;
use pjsh_core::{
    command::{Action, Args, Command, CommandResult},
    utils::path_to_string,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "source";
const NAME_SHORTHAND: &str = ".";

/// Execute commands from a file in the current shell.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct SourceOpts {
    /// Script file to execute.
    file: PathBuf,

    /// Script arguments.
    args: Vec<String>,
}

/// Implementation for the "source" built-in command.
#[derive(Clone)]
pub struct Source;
impl Command for Source {
    fn name(&self) -> &str {
        NAME
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match SourceOpts::try_parse_from(args.context.args()) {
            Ok(opts) => source_file(opts, args),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Implementation for the "." built-in command.
#[derive(Clone)]
pub struct SourceShorthand;

impl Command for SourceShorthand {
    fn name(&self) -> &str {
        NAME_SHORTHAND
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match SourceOpts::try_parse_from(args.context.args()) {
            Ok(opts) => source_file(opts, args),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Sources a file within a [`Context`].
fn source_file(opts: SourceOpts, args: &mut Args) -> CommandResult {
    if !opts.file.is_file() {
        let path = path_to_string(&opts.file);
        let _ = writeln!(args.io.stderr, "{NAME}: No such file: {}", path);
        return CommandResult::code(status::GENERAL_ERROR);
    }

    // Replace the context's arguments so that the file can be sourced using specific arguments.
    // This operation is destructive, so the original arguments must be stored temporarily in order
    // to restore the context before exiting.
    let mut args = Vec::with_capacity(opts.args.len() + 1);
    let source_file_name = opts
        .file
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("pjsh"));
    args.push(source_file_name);
    args.extend(opts.args);

    // A command does not have direct access to the mechanics of parsing and
    // executing a file. Thus, this must be performed using a shell action.
    let action = Action::SourceFile(opts.file, args);
    CommandResult::with_actions(status::SUCCESS, vec![action])
}
