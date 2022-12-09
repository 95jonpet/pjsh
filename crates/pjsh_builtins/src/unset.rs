use clap::Parser;
use pjsh_core::{
    command::{Args, Command, CommandResult},
    Context,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "unset";

/// Type to unset.
///
/// Determines what type of name should be unset.
#[derive(Clone, clap::ValueEnum)]
enum UnsetType {
    /// Treat each name as a shell function name.
    Function,
    /// Treat each name as a shell variable name.
    Variable,
}

/// Unset shell variables and/or functions.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct UnsetOpts {
    /// Determines whether to treat each name as a function or variable name.
    #[clap(value_enum, default_value = "variable", short, long)]
    r#type: UnsetType,

    /// Variable or function names to unset.
    #[clap(required = true, num_args = 1..)]
    name: Vec<String>,
}

/// Implementation for the "unset" built-in command.
#[derive(Clone)]
pub struct Unset;
impl Command for Unset {
    fn name(&self) -> &str {
        NAME
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match UnsetOpts::try_parse_from(args.context.args()) {
            Ok(opts) => unset_names(opts, args.context),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Unsets a collection of names in a context.
///
/// Returns an exit code.
fn unset_names(opts: UnsetOpts, ctx: &mut Context) -> CommandResult {
    match opts.r#type {
        UnsetType::Function => opts.name.iter().for_each(|f| ctx.unregister_function(f)),
        UnsetType::Variable => todo!("unset variable"), // opts.name.iter().for_each(|name| ctx.scope.unset_env(name)),
    };

    CommandResult::code(status::SUCCESS)
}
