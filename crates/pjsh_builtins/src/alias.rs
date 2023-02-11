use clap::Parser;
use pjsh_core::{
    command::{Args, Io},
    command::{Command, CommandResult},
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

    fn run(&self, args: &mut Args) -> CommandResult {
        match AliasOpts::try_parse_from(args.context.args()) {
            Ok(opts) => match (opts.name, opts.value) {
                (None, None) => display_aliases(args),
                (Some(name), None) => display_alias(&name, args),
                (Some(name), Some(value)) => set_alias(args.context, name, value),
                (None, Some(_)) => unreachable!(),
            },
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Displays an alias with a given name if it is defined within a context.
/// Otherwise, an error message is printed to stdout.
///
/// Returns an exit code.
fn display_alias(name: &str, args: &mut Args) -> CommandResult {
    if let Some(value) = args.context.aliases.get(name) {
        print_alias(name, value, args.io);
        CommandResult::code(status::SUCCESS)
    } else {
        let _ = writeln!(args.io.stderr, "{NAME}: {name}: not found");
        CommandResult::code(status::GENERAL_ERROR)
    }
}

/// Displays all aliases that are defined within a context.
///
/// Returns an exit code.
fn display_aliases(args: &mut Args) -> CommandResult {
    // Aliases should be printed in alphabetical order based on their names.
    let mut aliases: Vec<(String, String)> = args
        .context
        .aliases
        .iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    aliases.sort_by(|(a_key, _), (b_key, _)| a_key.cmp(b_key));

    for (name, value) in aliases {
        print_alias(&name, &value, args.io);
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
fn set_alias(context: &mut Context, name: String, value: String) -> CommandResult {
    context.aliases.insert(name, value);
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
            Some(vec!["alias".into(), "ls".into()]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);
        ctx.aliases.insert("ls".into(), "ls -lah".into());
        let (mut io, mut stdout, mut stderr) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let alias = Alias {};

        if let CommandResult::Builtin(result) = alias.run(&mut args) {
            assert_eq!(result.code, 0);
            assert!(result.actions.is_empty());
            assert_eq!(&file_contents(&mut stdout), "alias ls \"ls -lah\"\n");
            assert_eq!(&file_contents(&mut stderr), "");
        } else {
            unreachable!()
        }
    }

    #[test]
    fn it_can_print_aliases() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["alias".into()]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);
        ctx.aliases.insert("x".into(), "xyz".into());
        ctx.aliases.insert("a".into(), "abc".into());
        let (mut io, mut stdout, mut stderr) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let alias = Alias {};

        if let CommandResult::Builtin(result) = alias.run(&mut args) {
            assert_eq!(result.code, 0);
            assert!(result.actions.is_empty());
            assert_eq!(
                &file_contents(&mut stdout),
                "alias a \"abc\"\nalias x \"xyz\"\n" // Should be sorted by name.
            );
            assert_eq!(&file_contents(&mut stderr), "");
        } else {
            unreachable!()
        }
    }

    #[test]
    fn it_can_define_an_alias() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["alias".into(), "name".into(), "value".into()]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);
        let (mut io, mut stdout, mut stderr) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let alias = Alias {};

        if let CommandResult::Builtin(result) = alias.run(&mut args) {
            assert_eq!(ctx.aliases.get("name"), Some(&"value".to_owned()));
            assert_eq!(result.code, 0);
            assert!(result.actions.is_empty());
            assert_eq!(&file_contents(&mut stdout), "");
            assert_eq!(&file_contents(&mut stderr), "");
        } else {
            unreachable!()
        }
    }
}
