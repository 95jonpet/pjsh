use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_builtins::status;
use pjsh_core::{find_in_path, utils::path_to_string, Context, InternalCommand, InternalIo};

struct Type;
impl InternalCommand for Type {
    fn name(&self) -> &str {
        "type"
    }

    fn run(&self, args: &[String], ctx: Arc<Mutex<Context>>, io: Arc<Mutex<InternalIo>>) -> i32 {
        if args.is_empty() {
            let _ = writeln!(io.lock().stderr, "type: usage: type name [name ...]");
            return status::BUILTIN_ERROR;
        }

        let mut exit = status::SUCCESS;
        for arg in args {
            if let Some(cmd) = builtin(arg) {
                let _ = writeln!(io.lock().stdout, "{} is a shell builtin", cmd.name());
                continue;
            }

            if let Some(alias) = ctx.lock().scope.get_alias(arg) {
                let _ = writeln!(io.lock().stdout, "{} is aliased to `{}'", arg, alias);
                continue;
            }

            match find_in_path(arg, &ctx.lock()) {
                Some(path) => {
                    let _ = writeln!(io.lock().stdout, "{} is {}", arg, path_to_string(&path));
                }
                None => {
                    let path_var = ctx.lock().scope.get_env("PATH").unwrap_or_default();
                    let _ = writeln!(io.lock().stderr, "type: no {} in ({})", &arg, path_var);
                    exit = status::GENERAL_ERROR;
                }
            }
        }

        exit
    }
}

struct Which;
impl InternalCommand for Which {
    fn name(&self) -> &str {
        "which"
    }

    fn run(&self, args: &[String], ctx: Arc<Mutex<Context>>, io: Arc<Mutex<InternalIo>>) -> i32 {
        if args.is_empty() {
            let _ = writeln!(io.lock().stderr, "which: usage: which name [name ...]");
            return status::BUILTIN_ERROR;
        }

        let mut exit = status::SUCCESS;
        for arg in args {
            match find_in_path(arg, &ctx.lock()) {
                Some(path) => {
                    let _ = writeln!(io.lock().stdout, "{}", path_to_string(&path));
                }
                None => {
                    let path_var = ctx.lock().scope.get_env("PATH").unwrap_or_default();
                    let _ = writeln!(io.lock().stderr, "which: no {} in ({})", &arg, path_var);
                    exit = status::GENERAL_ERROR;
                }
            }
        }

        exit
    }
}

/// Returns a built-in [`InternalCommand`] with a given `name`.
pub fn builtin(name: &str) -> Option<Box<dyn InternalCommand>> {
    match name {
        "type" => Some(Box::new(Type {})),
        "which" => Some(Box::new(Which {})),
        name => pjsh_builtins::builtin(name),
    }
}
