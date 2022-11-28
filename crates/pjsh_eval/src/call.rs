use std::{
    collections::{HashMap, HashSet},
    io::Read,
    io::Write,
    path::Path,
    process,
};

use pjsh_ast::Function;
use pjsh_core::{
    command::{Args, Command, CommandResult, Io},
    Context, Scope, FD_STDERR, FD_STDIN, FD_STDOUT,
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
    let mut stdin: Box<dyn Read + Send> = Box::new(std::io::empty());
    let mut stdout: Box<dyn Write + Send> = Box::new(std::io::sink());
    let mut stderr: Box<dyn Write + Send> = Box::new(std::io::sink());

    if let Some(Ok(fd)) = context.reader(FD_STDIN) {
        stdin = fd;
    }
    if let Some(Ok(fd)) = context.writer(FD_STDOUT) {
        stdout = fd;
    }
    if let Some(Ok(fd)) = context.writer(FD_STDERR) {
        stderr = fd;
    }

    let mut io = Io::new(stdin, stdout, stderr);
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
    if let Some(path) = context.get_var("PWD") {
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
) -> EvalResult<()> {
    let function_args = &args[1..]; // The first argument is the function name.

    // Ensure that values are provided for all named arguments.
    if function_args.len() < function.args.len() {
        return Err(EvalError::UndefinedFunctionArguments(
            function.args[args.len()..].to_vec(),
        ));
    }

    if function_args.len() > function.args.len() {
        return Err(EvalError::UnboundFunctionArguments(
            function_args[function.args.len()..].to_vec(),
        ));
    }

    // Construct a temporary scope for the function body.
    let vars = HashMap::from_iter(function.args.iter().cloned().zip(args.iter().cloned()));
    context.push_scope(Scope::new(
        function.name.clone(),
        Some(Vec::from(args)),
        Some(vars),
        Some(HashMap::new()),
        HashSet::new(),
        false,
    ));

    let result = execute_statements(&function.body.statements, context);

    context.pop_scope();

    result
}
