use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_core::{Context, InternalCommand, InternalIo};

use crate::status;

#[derive(Clone)]
pub struct Unset;
impl InternalCommand for Unset {
    fn name(&self) -> &str {
        "unset"
    }

    /// Removes all environment variables with keys defined in `args`.
    fn run(
        &self,
        args: &[String],
        context: Arc<Mutex<Context>>,
        io: Arc<Mutex<InternalIo>>,
    ) -> i32 {
        if args.is_empty() {
            let _ = writeln!(io.lock().stderr, "unset: missing keys to unset");
            return status::BUILTIN_ERROR;
        }

        for arg in args {
            context.lock().scope.unset_env(arg);
        }

        status::SUCCESS
    }
}
