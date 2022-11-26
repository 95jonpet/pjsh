pub(crate) mod context;
pub(crate) mod file_buffer_shell;
pub(crate) mod interactive;
pub(crate) mod single_command_shell;
pub(crate) mod utils;

#[cfg(test)]
use mockall::automock;

pub(crate) enum ShellInput {
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
pub(crate) trait Shell {
    /// Prompts the user for a line of input using a `prompt` text that may contain ANSI control
    /// sequences.
    fn prompt_line(&mut self, prompt: &str) -> ShellInput;

    /// Returns `true` if the prompt is run interactively, i.e. the user can be prompted for
    /// additional input.
    fn is_interactive(&self) -> bool;

    /// Appends a line entry to the shell's history.
    ///
    /// Previous entries may be removed if the history is limited in size.
    ///
    /// This feature is optional to implement, and may be a no-op.
    fn add_history_entry(&mut self, line: &str);

    /// Saves the shell's history to a file.
    ///
    /// This feature is optional to implement, and may be a no-op.
    fn save_history(&mut self, history_file: &std::path::Path);
}
