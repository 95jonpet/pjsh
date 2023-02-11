use clap::Parser;
use pjsh_core::{
    command::{Args, Command, CommandResult},
    Context, Value,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "export";

/// Export variables from the shell's environment.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct ExportOpts {
    /// Variables to export.
    #[clap(required = true, num_args = 1..)]
    variables: Vec<String>,
}

/// Implementation for the "export" built-in command.
#[derive(Clone)]
pub struct Export;
impl Command for Export {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, args: &mut Args) -> CommandResult {
        match ExportOpts::try_parse_from(args.context.args()) {
            Ok(opts) => export_variables(opts, args),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Exports shell variables.
///
/// Returns 0 if all variables can be exported successfully, or 1 if at least
/// one argument cannot be exported.
fn export_variables(opts: ExportOpts, args: &mut Args) -> CommandResult {
    let mut result = CommandResult::code(status::SUCCESS);

    for variable in opts.variables {
        if let Err(err) = export_variable(variable, args.context) {
            let _ = writeln!(args.io.stderr, "{err}");
            result = CommandResult::code(status::GENERAL_ERROR);
        }
    }

    result
}

/// Exports a shell variable.
fn export_variable(variable: String, context: &mut Context) -> Result<(), String> {
    match variable.find('=') {
        // If an equals sign is present, the value should be set prior to the export.
        Some(separator) => {
            let name = variable[..separator].to_owned();
            let value = variable[separator + 1..].to_owned(); // The separator is not included.
            context.set_var(name.clone(), Value::Word(value));
            context.export_var(name)
        }

        // If there is no equals sign, the variable must already be known by the shell.
        None => context.export_var(variable),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use pjsh_core::{Context, Scope};

    use crate::utils::empty_io;

    use super::*;

    #[test]
    fn it_prints_help() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["export".into(), "--help".into()]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);
        let mut io = empty_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let cmd = Export {};
        let CommandResult::Builtin(result) = cmd.run(&mut args) else {
            unreachable!();
        };

        assert_eq!(result.code, 0);
    }

    #[test]
    fn it_exports_variables() {
        let export = Export {};
        let mut ctx = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            Some(vec!["export".into(), "var1".into(), "var2".into()]),
            HashMap::from([
                ("var1".into(), Some(Value::Word("val1".into()))),
                ("var2".into(), Some(Value::Word("val2".into()))),
            ]),
            HashMap::default(),
            HashSet::default(),
        )]);
        let mut io = empty_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let CommandResult::Builtin(result) = export.run(&mut args) else {
            unreachable!()
        };

        assert_eq!(result.code, status::SUCCESS);
        assert!(result.actions.is_empty());

        assert_eq!(
            ctx.exported_vars(),
            HashMap::from([("var1", "val1"), ("var2", "val2")])
        );
    }

    #[test]
    fn it_sets_and_exports_variables() {
        let export = Export {};
        let mut ctx = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            Some(vec!["export".into(), "var=val".into()]),
            HashMap::default(), // No variables are known.
            HashMap::default(),
            HashSet::default(),
        )]);
        let mut io = empty_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let CommandResult::Builtin(result) = export.run(&mut args) else {
            unreachable!()
        };

        assert_eq!(result.code, status::SUCCESS);
        assert!(result.actions.is_empty());

        // The variable should be set and exported.
        assert_eq!(ctx.get_var("var"), Some(&Value::Word("val".into())));
        assert_eq!(ctx.exported_vars(), HashMap::from([("var", "val")]),);
    }

    #[test]
    fn it_sets_and_exports_empty_variables() {
        let export = Export {};
        let mut ctx = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            Some(vec!["export".into(), "var=".into()]),
            HashMap::default(), // No variables are known.
            HashMap::default(),
            HashSet::default(),
        )]);
        let mut io = empty_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let CommandResult::Builtin(result) = export.run(&mut args) else {
            unreachable!()
        };

        assert_eq!(result.code, status::SUCCESS);
        assert!(result.actions.is_empty());

        // The variable should be set and exported.
        assert_eq!(ctx.get_var("var"), Some(&Value::Word("".into())));
        assert_eq!(ctx.exported_vars(), HashMap::from([("var", "")]),);
    }

    #[test]
    fn it_sets_and_exports_variables_with_multiple_separators() {
        let export = Export {};
        let mut ctx = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            Some(vec!["export".into(), "var=key=val".into()]),
            HashMap::default(), // No variables are known.
            HashMap::default(),
            HashSet::default(),
        )]);
        let mut io = empty_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let CommandResult::Builtin(result) = export.run(&mut args) else {
            unreachable!()
        };

        assert_eq!(result.code, status::SUCCESS);
        assert!(result.actions.is_empty());

        // The variable should be set and exported.
        assert_eq!(ctx.get_var("var"), Some(&Value::Word("key=val".into())));
        assert_eq!(ctx.exported_vars(), HashMap::from([("var", "key=val")]),);
    }

    #[test]
    fn it_does_not_export_unknown_variables() {
        let export = Export {};
        let mut ctx = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            Some(vec!["export".into(), "var1".into(), "var2".into()]),
            HashMap::default(), // No variables are known.
            HashMap::default(),
            HashSet::default(),
        )]);
        let mut io = empty_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let CommandResult::Builtin(result) = export.run(&mut args) else {
            unreachable!()
        };

        assert_eq!(result.code, status::GENERAL_ERROR);
        assert!(result.actions.is_empty());

        assert_eq!(
            ctx.exported_vars(),
            HashMap::default(),
            "nothing should be exported"
        );
    }
}
