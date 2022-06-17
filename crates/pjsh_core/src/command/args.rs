use std::sync::Arc;

use parking_lot::Mutex;

use crate::{command::Io, Context};

/// Arguments that can be passed to a command.
pub struct Args {
    /// Execution context for the command.
    pub context: Arc<Mutex<Context>>,
    /// File descriptors that the command can use for input and output.
    pub io: Io,
}

impl Args {
    pub fn from_context(context: Context, io: Io) -> Self {
        Self {
            context: Arc::new(Mutex::new(context)),
            io,
        }
    }
}
