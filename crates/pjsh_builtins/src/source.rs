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
        .expect("the source file should be an existing file");
    args.push(source_file_name);
    args.extend(opts.args);

    // A command does not have direct access to the mechanics of parsing and
    // executing a file. Thus, this must be performed using a shell action.
    let action = Action::SourceFile(opts.file, args);
    CommandResult::with_actions(status::SUCCESS, vec![action])
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use pjsh_core::{command::BuiltinCommandResult, Context, Scope};
    use tempfile::NamedTempFile;

    use crate::utils::empty_io;

    use super::*;

    /// Runs tests for [`Source`] and [`SourceShorthand`].
    fn test(context: Context, func: impl Fn(&str, &BuiltinCommandResult)) {
        let cmd = Source;
        let mut io = empty_io();
        let mut ctx = context.try_clone().expect("context should be cloned");
        let mut args = Args::new(&mut ctx, &mut io);
        let CommandResult::Builtin(result) = cmd.run(&mut args) else {
            unreachable!("Built-ins always return specialized built-in results");
        };
        func(cmd.name(), &result);

        let cmd = SourceShorthand;
        let mut io = empty_io();
        let mut ctx = context;
        let mut args = Args::new(&mut ctx, &mut io);
        let CommandResult::Builtin(result) = cmd.run(&mut args) else {
            unreachable!("Built-ins always return specialized built-in results");
        };
        func(cmd.name(), &result);
    }

    #[test]
    fn it_prints_help() {
        let ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["source".into(), "--help".into()]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);

        test(ctx, |name, result| {
            assert_eq!(result.code, 0, "`{name} --help` should exit successfully");
        });
    }

    #[test]
    fn it_sources_files() -> std::io::Result<()> {
        let script_file = NamedTempFile::new()?;

        let ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["source".into(), path_to_string(script_file.path())]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);

        test(ctx, |name, result| {
            assert_eq!(result.code, 0, "`{name} FILE` should exit successfully");
            assert!(
                matches!(
                &result.actions[0],
                Action::SourceFile(p, _) if p == script_file.path()),
                "`{name} FILE` should return an action to source FILE"
            );
        });
        Ok(())
    }

    #[test]
    fn it_errors_on_missing_file_to_source() -> std::io::Result<()> {
        let ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["source".into(), "/missing/file".into()]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);

        test(ctx, |name, result| {
            assert_ne!(
                result.code, 0,
                "`{name} MISSING_FILE` should not exit successfully"
            );
            assert!(
                result.actions.is_empty(),
                "`{name} MISSING_FILE` should not return an action to source MISSING_FILE"
            );
        });
        Ok(())
    }
}
