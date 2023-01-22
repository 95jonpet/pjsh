use pjsh_ast::Program;
use pjsh_core::Context;
use pjsh_eval::{execute_statement, EvalError};

use super::{ShellError, ShellResult};

/// Evaluates a program.
///
/// # Errors
///
/// If an error occurs during execution, the error handler is invoked.
///
/// If the error handler returns an error, execution is aborted
/// and the error is returned.
pub(crate) fn eval_program<ErrorHandler>(
    program: &Program,
    context: &mut Context,
    error_handler: ErrorHandler,
) -> ShellResult<()>
where
    ErrorHandler: Fn(EvalError) -> ShellResult<()>,
{
    for statement in &program.statements {
        if let Err(err) = execute_statement(statement, context) {
            error_handler(err)?;
        }
    }

    Ok(())
}

/// Prints an evaluation error.
pub(crate) fn print_error(error: EvalError) -> ShellResult<()> {
    eprintln!("pjsh: {error}");
    Ok(())
}

/// Returns a shell result wrapping an evaluation error.
pub(crate) fn exit_on_error(error: EvalError) -> ShellResult<()> {
    Err(ShellError::EvalError(error))
}
