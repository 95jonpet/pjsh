pub(crate) mod command;
pub(crate) mod file;
pub(crate) mod interactive;

#[cfg(test)]
use mockall::automock;

pub enum ShellInput {
    /// A line of input.
    Line(String),
    /// Interrupt the current process.
    Interrupt,
    /// Exit the shell.
    Logout,
    /// No input.
    None,
}

#[cfg_attr(test, automock)]
pub trait Shell {
    /// Prompts the user for a line of input using a `prompt` text that may contain ANSI control
    /// sequences.
    fn prompt_line(&mut self, prompt: &str) -> ShellInput;

    /// Returns `true` if the prompt is run interactively, i.e. the user can be prompted for
    /// additional input.
    fn is_interactive(&self) -> bool;

    /// Appends a line entry to the shell's history.
    ///
    /// Previous entries may be removed if the history is limited in size.
    fn add_history_entry(&mut self, line: &str);
}
