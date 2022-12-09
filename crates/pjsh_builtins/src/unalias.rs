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
    names: Vec<String>,
}

/// Implementation for the "unalias" built-in command.
#[derive(Clone)]
pub struct Unalias;
impl Command for Unalias {
    fn name(&self) -> &str {
        NAME
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match UnaliasOpts::try_parse_from(args.context.args()) {
            Ok(opts) => remove_aliases(args.context, &opts.names),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Removes a collection of aliases from a context.
///
/// Any undefined aliases are ignored.
///
/// Returns an exit code.
fn remove_aliases(context: &mut Context, names: &[String]) -> CommandResult {
    for name in names {
        context.aliases.remove(name);
    }
    CommandResult::code(status::SUCCESS)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use pjsh_core::{Context, Scope};

    use crate::utils::{file_contents, mock_io};

    use super::*;

    /// Constructs a context.
    fn context(args: Vec<String>) -> Context {
        Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(args),
            Some(HashMap::default()),
            HashMap::default(),
            HashSet::default(),
            false,
        )])
    }

    #[test]
    fn it_can_remove_existing_aliases() {
        let mut ctx = context(vec!["unalias".into(), "existing".into()]);
        ctx.aliases.insert("existing".into(), "ext".into());
        let (mut io, _stdout, _stderr) = mock_io();
        let unalias = Unalias {};

        let mut args = Args::new(&mut ctx, &mut io);
        if let CommandResult::Builtin(result) = unalias.run(&mut args) {
            assert_eq!(result.code, status::SUCCESS);
            assert_eq!(ctx.aliases, HashMap::new());
        }
    }

    #[test]
    fn it_can_ignore_non_existing_aliases() {
        let mut ctx = context(vec!["unalias".into(), "missing".into()]);
        let (mut io, _stdout, _stderr) = mock_io();
        let unalias = Unalias {};

        let mut args = Args::new(&mut ctx, &mut io);
        if let CommandResult::Builtin(result) = unalias.run(&mut args) {
            assert_eq!(result.code, status::SUCCESS);
            assert_eq!(ctx.aliases, HashMap::new());
        }
    }

    #[test]
    fn it_can_print_help() {
        let mut ctx = context(vec!["unalias".into(), "--help".into()]);
        let (mut io, mut stdout, _stderr) = mock_io();
        let unalias = Unalias {};

        let mut args = Args::new(&mut ctx, &mut io);
        if let CommandResult::Builtin(result) = unalias.run(&mut args) {
            assert_eq!(result.code, status::SUCCESS);
            assert_ne!(file_contents(&mut stdout), String::new());
        }
    }
}
