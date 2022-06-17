use clap::Parser;
use pjsh_core::{
    command::Io,
    command::{Args, Command, CommandResult},
    Context,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "alias";

/// Define or display aliases.
///
/// If called without any arguments, alias prints a list of all aliases.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct AliasOpts {
    /// Optional name of the alias to display or define.
    name: Option<String>,

    /// Optional alias value to define.
    value: Option<String>,
}

/// Implementation for the "alias" built-in command.
#[derive(Clone)]
pub struct Alias;
impl Command for Alias {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, mut args: Args) -> CommandResult {
        let mut ctx = args.context.lock();
        match AliasOpts::try_parse_from(ctx.args()) {
            Ok(opts) => match (opts.name, opts.value) {
                (None, None) => display_aliases(&ctx, &mut args.io),
                (Some(name), None) => display_alias(&name, &ctx, &mut args.io),
                (Some(name), Some(value)) => set_alias(name, value, &mut ctx),
                (None, Some(_)) => unreachable!(),
            },
            Err(error) => utils::exit_with_parse_error(&mut args.io, error),
        }
    }
}

/// Displays an alias with a given name if it is defined within a context.
/// Otherwise, an error message is printed to stdout.
///
/// Returns an exit code.
fn display_alias(name: &str, ctx: &Context, io: &mut Io) -> CommandResult {
    match ctx.aliases.get(name) {
        Some(value) => {
            print_alias(name, value, io);
            CommandResult::code(status::SUCCESS)
        }
        None => {
            let _ = writeln!(io.stderr, "{NAME}: {name}: not found");
            CommandResult::code(status::GENERAL_ERROR)
        }
    }
}

/// Displays all aliases that are defined within a context.
///
/// Returns an exit code.
fn display_aliases(ctx: &Context, io: &mut Io) -> CommandResult {
    // Aliases should be printed in alphabetical order based on their names.
    let mut aliases: Vec<(String, String)> = ctx
        .aliases
        .iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    aliases.sort_by(|(a_key, _), (b_key, _)| a_key.cmp(b_key));

    for (name, value) in aliases {
        print_alias(&name, &value, io);
    }
    CommandResult::code(status::SUCCESS)
}

/// Prints an alias to stdout.
fn print_alias(name: &str, value: &str, io: &mut Io) {
    if let Err(error) = writeln!(io.stdout, "alias {name} \"{value}\"") {
        let _ = writeln!(io.stderr, "{NAME}: unable to write to stdout: {error}");
    }
}

/// Sets an alias within a context.
///
/// Returns an exit code.
fn set_alias(name: String, value: String, ctx: &mut Context) -> CommandResult {
    ctx.aliases.insert(name, value);
    CommandResult::code(status::SUCCESS)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use pjsh_core::{Context, Scope};

    use crate::utils::{file_contents, mock_io};

    use super::*;

    #[test]
    fn it_can_print_a_matching_alias() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["alias".into(), "ls".into()],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        ctx.aliases.insert("ls".into(), "ls -lah".into());
        let (io, mut stdout, mut stderr) = mock_io();

        let alias = Alias {};

        let args = Args::from_context(ctx, io);
        let result = alias.run(args);

        assert_eq!(result.code, 0);
        assert!(result.actions.is_empty());
        assert_eq!(&file_contents(&mut stdout), "alias ls \"ls -lah\"\n");
        assert_eq!(&file_contents(&mut stderr), "");
    }

    #[test]
    fn it_can_print_aliases() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["alias".into()],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        ctx.aliases.insert("x".into(), "xyz".into());
        ctx.aliases.insert("a".into(), "abc".into());
        let (io, mut stdout, mut stderr) = mock_io();

        let alias = Alias {};

        let args = Args::from_context(ctx, io);
        let result = alias.run(args);

        assert_eq!(result.code, 0);
        assert!(result.actions.is_empty());
        assert_eq!(
            &file_contents(&mut stdout),
            "alias a \"abc\"\nalias x \"xyz\"\n" // Should be sorted by name.
        );
        assert_eq!(&file_contents(&mut stderr), "");
    }

    #[test]
    fn it_can_define_an_alias() {
        let ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["alias".into(), "name".into(), "value".into()],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        let (io, mut stdout, mut stderr) = mock_io();

        let alias = Alias {};

        let args = Args::from_context(ctx, io);
        let ctx = args.context.clone();
        let result = alias.run(args);

        assert_eq!(ctx.lock().aliases.get("name"), Some(&"value".to_owned()));
        assert_eq!(result.code, 0);
        assert!(result.actions.is_empty());
        assert_eq!(&file_contents(&mut stdout), "");
        assert_eq!(&file_contents(&mut stderr), "");
    }
}
