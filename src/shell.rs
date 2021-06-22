use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

use clap::crate_name;

/// Shell prompt during normal operation.
pub const PS1: &str = "$ ";

/// Shell prompt when a logical line should be continued.
pub const PS2: &str = "> ";

pub enum Lines {
    Buffered(Box<dyn BufRead>),
    Single(Option<String>),
}

impl Iterator for Lines {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        match self {
            Self::Buffered(reader) => {
                let mut buffer = String::new();
                match reader.read_line(&mut buffer) {
                    Ok(0) | Err(_) => None,
                    _ => Some(buffer),
                }
            }
            Self::Single(maybe_string) => match maybe_string {
                Some(string) => {
                    let clone = Some(string.clone());
                    *maybe_string = None;
                    clone
                }
                None => None,
            },
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
    /// Wraps a command in a [`Shell`] for execution.
    pub fn from_command(command: String) -> Self {
        Self {
            lines: Lines::Single(Some(command)),
            interactive: false,
            name: String::from(crate_name!()),
            vars: HashMap::new(),
        }
    }

    /// Instantiates a [`Shell`] for executing a file.
    pub fn from_file(path: PathBuf) -> Self {
        Self {
            lines: Lines::Buffered(Box::new(BufReader::new(fs::File::open(&path).unwrap()))),
            interactive: false,
            name: String::from(path.file_name().unwrap().to_str().unwrap()),
            vars: HashMap::new(),
        }
    }

    pub fn get_var(&self, key: &str) -> Option<String> {
        self.vars
            .get(key)
            .map_or(env::var(key).ok(), |s| Some(String::from(s)))
    }

    /// Instantiates an interactive [`Shell`].
    pub fn interactive() -> Self {
        Self {
            lines: Lines::Buffered(Box::new(BufReader::new(io::stdin()))),
            interactive: true,
            name: String::from(crate_name!()),
            vars: HashMap::new(),
        }
    }

    /// Returns true if the shell is interactive.
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    /// Returns the next input from a prompt.
    pub fn next_prompt(&mut self, prompt: &str) -> Option<String> {
        if self.is_interactive() {
            print!("{}", prompt);
            io::stdout().flush().unwrap();
        }
        self.lines.next()
    }
}

impl Iterator for Shell {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.next_prompt(PS1)
    }
}
