use super::{args::Args, CommandResult};

/// A command is something that can be executed by the shell.
pub trait Command: CommandClone + Send + Sync {
    /// Returns the command's name.
    fn name(&self) -> &str;

    /// Runs the command.
    fn run(&self, args: Args) -> CommandResult;
}

/// Helper trait for making it easier to clone `Box<Command>`.
pub trait CommandClone {
    fn clone_box(&self) -> Box<dyn Command>;
}

impl<T> CommandClone for T
where
    T: 'static + Command + Clone,
{
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Command> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
