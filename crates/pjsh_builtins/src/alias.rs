use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_core::{Context, InternalCommand, InternalIo};

use crate::status;

#[derive(Clone)]
pub struct Alias;
impl InternalCommand for Alias {
    fn name(&self) -> &str {
        "alias"
    }

    fn run(
        &self,
        args: &[String],
        context: Arc<Mutex<Context>>,
        io: Arc<Mutex<InternalIo>>,
    ) -> i32 {
        match args {
            [] => {
                for (key, value) in context.lock().scope.aliases() {
                    let _ = writeln!(io.lock().stdout, "alias {} = '{}'", &key, &value);
                }
                status::SUCCESS
            }
            [key, op, value] if op == "=" => {
                context.lock().scope.set_alias(key.clone(), value.clone());
                status::SUCCESS
            }
            [_, op] if op == "=" => {
                let _ = writeln!(
                    io.lock().stderr,
                    "alias: invalid arguments: {}",
                    args.join(" ")
                );
                status::BUILTIN_ERROR
            }
            keys => {
                let mut exit = status::SUCCESS;
                for key in keys {
                    if let Some(value) = context.lock().scope.get_alias(key) {
                        let _ = writeln!(io.lock().stdout, "alias {} = '{}'", &key, &value);
                    } else {
                        let _ = writeln!(io.lock().stderr, "alias: {}: not found", &key);
                        exit = status::GENERAL_ERROR;
                    }
                }
                exit
            }
        }
    }
}

#[derive(Clone)]
pub struct Unalias;
impl InternalCommand for Unalias {
    fn name(&self) -> &str {
        "unalias"
    }

    fn run(
        &self,
        args: &[String],
        context: Arc<Mutex<Context>>,
        io: Arc<Mutex<InternalIo>>,
    ) -> i32 {
        if args.is_empty() {
            let _ = writeln!(io.lock().stderr, "unalias: usage: unalias name [name ...]");
            return status::BUILTIN_ERROR;
        }

        for arg in args {
            context.lock().scope.unset_alias(arg);
        }

        status::SUCCESS
    }
}
