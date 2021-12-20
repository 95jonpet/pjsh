use std::io;

use crate::Context;

pub struct InternalIo {
    pub stdin: Box<dyn io::Read>,
    pub stdout: Box<dyn io::Write>,
    pub stderr: Box<dyn io::Write>,
}

impl InternalIo {
    pub fn new(
        stdin: Box<dyn io::Read>,
        stdout: Box<dyn io::Write>,
        stderr: Box<dyn io::Write>,
    ) -> Self {
        Self {
            stdin,
            stdout,
            stderr,
        }
    }
}

pub trait InternalCommand: Send + Sync {
    /// Returns the command's name.
    fn name(&self) -> &str;

    /// Runs the command.
    ///
    /// Returns an exit status. Only the last 8 bits of information are guaranteed to be useful.
    fn run(&self, args: &[String], context: &mut Context, io: &mut InternalIo) -> i32;
}
