use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_ast::{FileDescriptor::*, Redirect};
use pjsh_core::{utils::resolve_path, Context, FileDescriptor};

use crate::{interpolate::interpolate_word, EvalError, EvalResult};

pub(crate) fn handle_redirects(redirects: &[Redirect], ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    for redirect in redirects {
        handle_redirect(redirect, Arc::clone(&ctx))?;
    }
    Ok(())
}

fn handle_redirect(redirect: &Redirect, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    match (redirect.source.clone(), redirect.target.clone()) {
        (Number(from), Number(to)) => {
            let mut ctx = ctx.lock();
            if let Some(fd) = ctx.files.get(&to).cloned() {
                ctx.files.insert(from, fd);
            } else {
                return Err(EvalError::UnknownFileDescriptor(to.to_string()));
            }
        }
        (Number(source), File(file_path)) => {
            if let Ok(file_path) = interpolate_word(&file_path, Arc::clone(&ctx)) {
                let path = resolve_path(&ctx.lock(), &file_path);
                match redirect.operator {
                    pjsh_ast::RedirectOperator::Write => {
                        ctx.lock().files.insert(source, FileDescriptor::File(path));
                    }
                    pjsh_ast::RedirectOperator::Append => {
                        ctx.lock()
                            .files
                            .insert(source, FileDescriptor::AppendFile(path));
                    }
                }
            } else {
                return Err(EvalError::InvalidRedirect(redirect.clone()));
            }
        }
        (File(file_path), Number(target)) => {
            if let Ok(file_path) = interpolate_word(&file_path, Arc::clone(&ctx)) {
                let path = resolve_path(&ctx.lock(), &file_path);
                ctx.lock().files.insert(target, FileDescriptor::File(path));
            } else {
                return Err(EvalError::InvalidRedirect(redirect.clone()));
            }
        }
        (File(_), File(_)) => {
            unreachable!("cannot redirect input from file to file");
        }
    }

    Ok(())
}
