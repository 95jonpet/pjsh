use std::io::{BufRead, BufReader, Read};

use crate::shell::{Shell, ShellInput};

/// A minimal non-interactive shell reading input from a type implementing
/// [`std::io::Read`].
///
/// Input is buffered internally in order to increase read performance.
pub struct InputShell<R: Read> {
    /// Internal read buffer.
    reader: BufReader<R>,
}

impl<R: Read> InputShell<R> {
    /// Constructs a new shell from a type implementing [`std::io::Read`].
    pub fn new(input: R) -> Self {
        let reader = BufReader::new(input);
        Self { reader }
    }
}

impl<R: Read> Shell for InputShell<R> {
    fn prompt_line(&mut self, _prompt: &str) -> ShellInput {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) | Err(_) => ShellInput::None,
            _ => ShellInput::Line(line),
        }
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
