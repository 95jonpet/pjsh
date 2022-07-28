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

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    use parking_lot::Mutex;
    use pjsh_core::Scope;

    use crate::utils::{file_contents, mock_io};

    use super::*;

    /// Constructs a context.
    fn context(args: Vec<String>) -> Arc<Mutex<Context>> {
        let context = Context::with_scopes(vec![Scope::new(
            String::new(),
            args,
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        Arc::new(Mutex::new(context))
    }

    #[test]
    fn it_can_remove_existing_aliases() {
        let ctx = context(vec!["unalias".into(), "existing".into()]);
        ctx.lock().aliases.insert("existing".into(), "ext".into());
        let (io, _stdout, _stderr) = mock_io();
        let unset = Unalias {};

        let args = Args::new(Arc::clone(&ctx), io);
        let result = unset.run(args);

        assert_eq!(result.code, status::SUCCESS);
        assert_eq!(ctx.lock().aliases, HashMap::new());
    }

    #[test]
    fn it_can_ignore_non_existing_aliases() {
        let ctx = context(vec!["unalias".into(), "missing".into()]);
        let (io, _stdout, _stderr) = mock_io();
        let unset = Unalias {};

        let args = Args::new(Arc::clone(&ctx), io);
        let result = unset.run(args);

        assert_eq!(result.code, status::SUCCESS);
        assert_eq!(ctx.lock().aliases, HashMap::new());
    }

    #[test]
    fn it_can_print_help() {
        let ctx = context(vec!["unalias".into(), "--help".into()]);
        let (io, mut stdout, _stderr) = mock_io();
        let unset = Unalias {};

        let args = Args::new(Arc::clone(&ctx), io);
        let result = unset.run(args);

        assert_eq!(result.code, status::SUCCESS);
        assert_ne!(file_contents(&mut stdout), String::new());
    }
}
