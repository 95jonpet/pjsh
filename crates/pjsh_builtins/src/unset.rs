use clap::Parser;
use pjsh_core::{
    command::{Args, Command, CommandResult},
    Context,
};

use crate::utils;

/// Command name.
const NAME: &str = "unset";

/// Type to unset.
///
/// Determines what type of name should be unset.
#[derive(Clone, clap::ArgEnum)]
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
    #[clap(arg_enum, default_value = "variable", short, long)]
    r#type: UnsetType,

    /// Variable or function names to unset.
    #[clap(required = true, min_values = 1)]
    name: Vec<String>,
}

/// Implementation for the "unset" built-in command.
#[derive(Clone)]
pub struct Unset;
impl Command for Unset {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, mut args: Args) -> CommandResult {
        let mut ctx = args.context.lock();
        match UnsetOpts::try_parse_from(ctx.args()) {
            Ok(opts) => unset_names(opts, &mut ctx),
            Err(error) => utils::exit_with_parse_error(&mut args.io, error),
        }
    }
}

/// Unsets a collection of names in a context.
///
/// Returns an exit code.
fn unset_names(opts: UnsetOpts, _ctx: &mut Context) -> CommandResult {
    match opts.r#type {
        UnsetType::Function => todo!("unset function"),
        UnsetType::Variable => todo!("unset variable"), // opts.name.iter().for_each(|name| ctx.scope.unset_env(name)),
    };

    // CommandResult::code(status::SUCCESS)
}
