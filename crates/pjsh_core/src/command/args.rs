use crate::{command::Io, Context};

/// Arguments that can be passed to a command.
pub struct Args<'a> {
    /// Execution context for the command.
    pub context: &'a mut Context,

    /// File descriptors that the command can use for input and output.
    pub io: &'a mut Io,
}

impl<'a> Args<'a> {
    /// Constructs a new command argument wrapper.
    pub fn new(context: &'a mut Context, io: &'a mut Io) -> Self {
        Self { context, io }
    }
}
