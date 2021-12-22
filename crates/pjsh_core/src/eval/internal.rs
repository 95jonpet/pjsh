use std::{io, sync::Arc};

use parking_lot::Mutex;

use crate::Context;

pub struct InternalIo {
    pub stdin: Box<dyn io::Read + Send>,
    pub stdout: Box<dyn io::Write + Send>,
    pub stderr: Box<dyn io::Write + Send>,
}

impl InternalIo {
    pub fn new(
        stdin: Box<dyn io::Read + Send>,
        stdout: Box<dyn io::Write + Send>,
        stderr: Box<dyn io::Write + Send>,
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
    fn run(&self, args: &[String], context: Arc<Mutex<Context>>, io: Arc<Mutex<InternalIo>>)
        -> i32;
}
