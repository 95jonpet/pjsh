use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_builtins::status;
use pjsh_core::{Context, InternalCommand, InternalIo};
use pjsh_exec::interpolate_word;
use pjsh_parse::parse_interpolation;

use crate::{exec::create_executor, run_shell, shell::file::FileBufferShell};

#[derive(Clone)]
pub(crate) struct Interpolate;
impl InternalCommand for Interpolate {
    fn name(&self) -> &str {
        "interpolate"
    }

    fn run(&self, args: &[String], ctx: Arc<Mutex<Context>>, io: Arc<Mutex<InternalIo>>) -> i32 {
        if args.is_empty() {
            let _ = writeln!(
                io.lock().stderr,
                "interpolate: usage: interpolate: text [text ...]"
            );
            return status::BUILTIN_ERROR;
        }

        let executor = create_executor();
        let mut exit = status::SUCCESS;
        for arg in args {
            let interpolated_value =
                parse_interpolation(arg).map(|word| interpolate_word(&executor, word, &ctx.lock()));
            match interpolated_value {
                Ok(value) => {
                    let _ = writeln!(io.lock().stdout, "{}", &value);
                }
                Err(error) => {
                    let _ = writeln!(io.lock().stderr, "interpolate: {}", error);
                    exit = status::GENERAL_ERROR;
                }
            }
        }

        exit
    }
}

#[derive(Clone)]
pub(crate) struct Source;
impl InternalCommand for Source {
    fn name(&self) -> &str {
        "source"
    }

    fn run(&self, args: &[String], ctx: Arc<Mutex<Context>>, io: Arc<Mutex<InternalIo>>) -> i32 {
        if args.is_empty() {
            let _ = writeln!(
                io.lock().stderr,
                "source: usage: source: filename [argument ...]"
            );
            return status::BUILTIN_ERROR;
        }

        if args.len() > 1 {
            todo!("support arguments when sourcing files");
        }

        let script_file = args.get(0).expect("script file argument");
        let shell = FileBufferShell::new(script_file);

        run_shell(Box::new(shell), Arc::clone(&ctx));
        ctx.lock().last_exit
    }
}
