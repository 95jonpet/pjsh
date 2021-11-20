use super::Shell;

pub struct SingleCommandShell {
    it: Option<String>,
}

impl SingleCommandShell {
    pub fn new(line: String) -> Self {
        Self { it: Some(line) }
    }
}

impl Shell for SingleCommandShell {
    fn prompt_line(&mut self, _prompt: &str) -> Option<String> {
        std::mem::take(&mut self.it)
    }

    fn add_history_entry(&mut self, _line: &str) {
        // Intentionally left blank.
    }

    fn is_interactive(&self) -> bool {
        false
    }
}
