use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};

static PS1: &str = "$";
// static PS2: &str = ">> ";

pub struct Lines {
    buffer: Box<dyn BufRead>,
}

impl Lines {
    pub fn new(buffer: Box<dyn BufRead>) -> Self {
        Self { buffer }
    }
}

impl Iterator for Lines {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut buffer = String::new();
        match self.buffer.read_line(&mut buffer) {
            Ok(0) | Err(_) => None,
            _ => Some(buffer),
        }
    }
}

pub struct Shell {
    lines: Lines,
    interactive: bool,
    #[allow(dead_code)]
    name: String,
    pub vars: HashMap<String, String>,
}

impl Shell {
    pub fn new(file: Option<String>) -> Self {
        if let Some(file_path) = file {
            Self {
                lines: Lines::new(Box::new(BufReader::new(
                    fs::File::open(&file_path).unwrap(),
                ))),
                interactive: false,
                name: file_path,
                vars: HashMap::new(),
            }
        } else {
            Self {
                lines: Lines::new(Box::new(BufReader::new(io::stdin()))),
                interactive: true,
                name: String::from("pjsh"),
                vars: HashMap::new(),
            }
        }
    }

    pub fn is_interactive(&self) -> bool {
        self.interactive
    }
}

impl Iterator for Shell {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.interactive {
            print!("{} ", PS1);
            io::stdout().flush().unwrap();
        }
        self.lines.next()
    }
}
