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
        UnsetType::Variable => opts.name.iter().for_each(|v| ctx.unset_var(v)),
    };

    CommandResult::code(status::SUCCESS)
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use pjsh_ast::{Block, Function};
    use pjsh_core::{Scope, Value};

    use crate::utils::mock_io;

    use super::*;

    #[test]
    fn it_prints_help() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["unset".into(), "--help".into()]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);
        let (mut io, _, _) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let cmd = Unset {};
        let CommandResult::Builtin(result) = cmd.run(&mut args) else {
            unreachable!();
        };

        assert_eq!(result.code, 0);
    }

    #[test]
    fn it_unsets_variables() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["unset".into(), "var".into()]),
            HashMap::from([("var".into(), Some(Value::Word("value".into())))]),
            HashMap::default(),
            HashSet::default(),
        )]);
        let (mut io, _, _) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let cmd = Unset {};
        let CommandResult::Builtin(result) = cmd.run(&mut args) else {
            unreachable!();
        };

        assert_eq!(result.code, 0);
        assert!(result.actions.is_empty());
        assert_eq!(ctx.get_var("var"), None);
    }

    #[test]
    fn it_unsets_functions() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec![
                "unset".into(),
                "--type=function".into(),
                "func".into(),
            ]),
            HashMap::default(),
            HashMap::from([(
                "func".into(),
                Some(Function {
                    name: "func".into(),
                    args: Vec::default(),
                    list_arg: None,
                    body: Block {
                        statements: Vec::default(),
                    },
                }),
            )]),
            HashSet::default(),
        )]);
        let (mut io, _, _) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let cmd = Unset {};
        let CommandResult::Builtin(result) = cmd.run(&mut args) else {
            unreachable!();
        };

        assert_eq!(result.code, 0);
        assert!(result.actions.is_empty());
        assert_eq!(ctx.get_function("func"), None);
    }
}
