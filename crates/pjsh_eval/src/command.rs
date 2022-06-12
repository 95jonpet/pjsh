use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_ast::Redirect;
use pjsh_core::Context;

use crate::{redirect::handle_redirects, EvalResult};

pub(crate) fn execute_command(
    args: &[String],
    redirects: &[Redirect],
    ctx: Arc<Mutex<Context>>,
) -> EvalResult<()> {
    assert!(!args.is_empty());

    handle_redirects(redirects, Arc::clone(&ctx))?;
    let command = &args[0];

    if let Some(_builtin) = ctx.lock().builtins.get(command) {
        todo!("execute built-in command");
    }

    if let Some(_function) = ctx.lock().get_function(command) {
        todo!("execute function");
    }

    execute_external_command(args, Arc::clone(&ctx))
}

fn execute_external_command(args: &[String], ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    let mut command = std::process::Command::new(args[0].to_owned());
    command.args(&args[1..]);

    let status = command.status().unwrap();
    ctx.lock().last_exit = status.code().unwrap_or(255); // TODO: What should the default error be?

    Ok(())
}
