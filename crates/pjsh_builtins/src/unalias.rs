use clap::Parser;
use pjsh_core::{
    command::{Args, Command, CommandResult},
    Context,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "unalias";

/// Remove aliases.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct UnaliasOpts {
    /// Aliases to remove.
    name: Vec<String>,
}

/// Implementation for the "unalias" built-in command.
#[derive(Clone)]
pub struct Unalias;
impl Command for Unalias {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, mut args: Args) -> CommandResult {
        let mut ctx = args.context.lock();
        match UnaliasOpts::try_parse_from(ctx.args()) {
            Ok(opts) => remove_aliases(&opts.name, &mut ctx),
            Err(error) => utils::exit_with_parse_error(&mut args.io, error),
        }
    }
}

/// Removes a collection of aliases from a context.
///
/// Any undefined aliases are ignored.
///
/// Returns an exit code.
fn remove_aliases(names: &[String], ctx: &mut Context) -> CommandResult {
    for name in names {
        ctx.aliases.remove(name);
    }
    CommandResult::code(status::SUCCESS)
}
