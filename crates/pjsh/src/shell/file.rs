use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
};

use super::Shell;

pub struct FileBufferShell {
    reader: BufReader<fs::File>,
}

impl FileBufferShell {
    pub fn new(script_file: impl AsRef<Path>) -> Self {
        let reader = BufReader::new(fs::File::open(script_file).unwrap());
        Self { reader }
    }
}

impl Shell for FileBufferShell {
    fn prompt_line(&mut self, _prompt: &str) -> Option<String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) | Err(_) => None,
            _ => Some(line),
        }
    }

    fn add_history_entry(&mut self, _line: &str) {
        // Intentionally left blank.
    }

    fn is_interactive(&self) -> bool {
        false
    }
}
