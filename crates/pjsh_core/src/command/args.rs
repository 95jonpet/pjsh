use std::slice::Iter;

use crate::{command::Io, Context};

/// Arguments that can be passed to a command.
pub struct Args {
    /// Execution context for the command.
    pub context: Context,
    /// File descriptors that the command can use for input and output.
    pub io: Io,
}

impl Args {
    /// Returns an `Iter<String>` over all arguments.
    ///
    /// Note that the first argument (index 0) contains the function or command
    /// name.
    pub fn iter(&self) -> Iter<String> {
        self.context.arguments.iter()
    }
}
