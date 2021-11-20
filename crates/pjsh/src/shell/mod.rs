pub(crate) mod command;
pub(crate) mod file;
pub(crate) mod interactive;

pub trait Shell {
    /// Prompts the user for a line of input using a `prompt` text that may contain ANSI control
    /// sequences.
    fn prompt_line(&mut self, prompt: &str) -> Option<String>;

    /// Returns `true` if the prompt is run interactively, i.e. the user can be prompted for
    /// additional input.
    fn is_interactive(&self) -> bool;

    fn add_history_entry(&mut self, line: &str);
}
