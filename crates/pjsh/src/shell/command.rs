use super::{Shell, ShellInput};

pub struct SingleCommandShell {
    it: Option<String>,
}

impl SingleCommandShell {
    pub fn new(line: String) -> Self {
        Self { it: Some(line) }
    }
}

impl Shell for SingleCommandShell {
    fn prompt_line(&mut self, _prompt: &str) -> ShellInput {
        if let Some(line) = std::mem::take(&mut self.it) {
            return ShellInput::Line(line);
        }

        ShellInput::None
    }

    fn is_interactive(&self) -> bool {
        false
    }

    fn add_history_entry(&mut self, _line: &str) {
        // Intentionally left blank.
    }

    fn save_history(&mut self, _path: &std::path::Path) {
        // Intentionally left blank.
    }
}
