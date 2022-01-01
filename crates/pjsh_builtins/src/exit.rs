use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_core::{Context, InternalCommand, InternalIo};

use crate::status;

const EXIT_INVALID_CODE: i32 = 128;

pub struct Exit;

impl InternalCommand for Exit {
    fn name(&self) -> &str {
        "exit"
    }

    fn run(&self, args: &[String], ctx: Arc<Mutex<Context>>, io: Arc<Mutex<InternalIo>>) -> i32 {
        match args {
            [] => ctx.lock().last_exit,
            [n] => n.parse::<i32>().unwrap_or(EXIT_INVALID_CODE),
            _ => {
                let _ = writeln!(io.lock().stderr, "exit: too many arguments");
                status::BUILTIN_ERROR
            }
        }
    }
}
