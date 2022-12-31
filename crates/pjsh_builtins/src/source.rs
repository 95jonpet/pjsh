use std::path::PathBuf;

use clap::Parser;
use pjsh_core::{
    command::{Args, Command, CommandResult},
    Context,
};

use crate::utils;

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
pub struct Source<F>
where
    F: Fn(PathBuf, &mut Context),
{
    /// Callback function for sourcing a file.
    source_function: F,
}

impl<F> Source<F>
where
    F: Fn(PathBuf, &mut Context),
{
    /// Constructs a new "source" built-in.
    pub fn new(source_function: F) -> Self {
        Self { source_function }
    }
}

impl<F> Command for Source<F>
where
    F: Fn(PathBuf, &mut Context) + Send + Sync + Clone + 'static,
{
    fn name(&self) -> &str {
        NAME
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match SourceOpts::try_parse_from(args.context.args()) {
            Ok(opts) => {
                let old_args = args.context.replace_args(Some(opts.args));
                (self.source_function)(opts.file, args.context);
                args.context.replace_args(old_args); // Restore args in context.
                CommandResult::code(args.context.last_exit())
            }
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Implementation for the "." built-in command.
#[derive(Clone)]
pub struct SourceShorthand<F>
where
    F: Fn(PathBuf, &mut Context),
{
    /// Callback function for sourcing a file.
    source_function: F,
}

impl<F> SourceShorthand<F>
where
    F: Fn(PathBuf, &mut Context),
{
    /// Constructs a new "source" built-in.
    pub fn new(source_function: F) -> Self {
        Self { source_function }
    }
}

impl<F> Command for SourceShorthand<F>
where
    F: Fn(PathBuf, &mut Context) + Send + Sync + Clone + 'static,
{
    fn name(&self) -> &str {
        NAME_SHORTHAND
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match SourceOpts::try_parse_from(args.context.args()) {
            Ok(opts) => {
                let old_args = args.context.replace_args(Some(opts.args));
                (self.source_function)(opts.file, args.context);
                args.context.replace_args(old_args); // Restore args in context.
                CommandResult::code(args.context.last_exit())
            }
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}
