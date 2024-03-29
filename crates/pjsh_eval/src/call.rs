use std::{
    collections::{HashMap, HashSet},
    path::Path,
    process,
};

use pjsh_ast::Function;
use pjsh_core::{
    command::{Args, Command, CommandResult},
    utils::word_var,
    Context, Scope, Value, FD_STDERR, FD_STDIN, FD_STDOUT,
};

use crate::{
    error::{EvalError, EvalResult},
    execute_statements,
};

/// Calls a built-in command.
pub fn call_builtin_command(
    command: &dyn Command,
    args: &[String],
    context: &mut Context,
) -> EvalResult<CommandResult> {
    let mut io = context.io();
    let original_args = context.replace_args(Some(args.to_vec()));
    let mut args = Args::new(context, &mut io);
    let result = command.run(&mut args);
    context.replace_args(original_args);
    Ok(result)
}

/// Returns a prepared call to an external using [`std::process::Command`].
pub fn call_external_program<P: AsRef<Path>>(
    program: P,
    args: &[String],
    context: &mut Context,
) -> EvalResult<process::Command> {
    let mut cmd = process::Command::new(program.as_ref());
    cmd.envs(context.exported_vars());
    cmd.args(args);

    // Spawn the new process within the context's working directory rather than that
    // of the current process.
    if let Some(path) = word_var(context, "PWD") {
        cmd.current_dir(path);
    }

    // The new process should inherit the context's file descriptors.
    if let Some(stdin) = context.input(FD_STDIN) {
        cmd.stdin(stdin.map_err(|e| EvalError::FileDescriptorError(FD_STDIN, e))?);
    }
    if let Some(stdout) = context.output(FD_STDOUT) {
        cmd.stdout(stdout.map_err(|e| EvalError::FileDescriptorError(FD_STDOUT, e))?);
    }
    if let Some(stderr) = context.output(FD_STDERR) {
        cmd.stderr(stderr.map_err(|e| EvalError::FileDescriptorError(FD_STDERR, e))?);
    }

    Ok(cmd)
}

/// Calls a function.
pub fn call_function(
    function: &Function,
    args: &[String],
    context: &mut Context,
) -> EvalResult<CommandResult> {
    let function_args = &args[1..]; // The first argument is the function name.

    // Ensure that values are provided for all named arguments.
    if function_args.len() < function.args.len() {
        return Err(EvalError::UndefinedFunctionArguments(
            function.args[args.len()..].to_vec(),
        ));
    }

    if function_args.len() > function.args.len() && function.list_arg.is_none() {
        return Err(EvalError::UnboundFunctionArguments(
            function_args[function.args.len()..].to_vec(),
        ));
    }

    // Construct a temporary scope for the function body.
    let mut vars = HashMap::from_iter(
        function
            .args
            .iter()
            .cloned()
            .zip(args.iter().cloned().map(Value::Word).map(Some)),
    );

    if let Some(list_arg_name) = &function.list_arg {
        let list_args = &args[function.args.len()..];
        vars.insert(
            list_arg_name.clone(),
            Some(Value::List(Vec::from(list_args))),
        );
    }

    context.push_scope(Scope::new(
        function.name.clone(),
        Some(Vec::from(args)),
        vars,
        HashMap::new(),
        HashSet::new(),
    ));

    let result = execute_statements(&function.body.statements, context);

    context.pop_scope();

    result.map(|_| CommandResult::code(0))
}

#[cfg(test)]
mod tests {
    use pjsh_core::FileDescriptor;

    use super::*;

    #[derive(Clone)]
    struct MyBuiltin;
    impl Command for MyBuiltin {
        fn name(&self) -> &str {
            "mybuiltin"
        }

        fn run(&self, _args: &mut Args) -> CommandResult {
            CommandResult::code(0)
        }
    }

    #[test]
    fn test_call_builtin_command() -> EvalResult<()> {
        let mut context = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            None,
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);

        context.set_file_descriptor(FD_STDIN, FileDescriptor::Null);
        context.set_file_descriptor(FD_STDOUT, FileDescriptor::Null);
        context.set_file_descriptor(FD_STDERR, FileDescriptor::Null);

        let command = MyBuiltin;

        let CommandResult::Builtin(result) =
            call_builtin_command(&command, &["mybuiltin".into()], &mut context)? else {
                unreachable!()
            };
        assert_eq!(result.code, 0);
        Ok(())
    }
}
