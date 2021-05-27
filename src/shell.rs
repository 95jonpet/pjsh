use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

use clap::crate_name;

static PS1: &str = "$";
// static PS2: &str = ">> ";

pub enum Lines {
    Buffered(Box<dyn BufRead>),
    Single(Option<String>),
}

// impl Lines {
//     pub fn new(buffer: Box<dyn BufRead>) -> Self {
//         Self { buffer }
//     }
// }

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
    pub fn from_command(command: String) -> Self {
        Self {
            lines: Lines::Single(Some(command)),
            interactive: false,
            name: String::from(crate_name!()),
            vars: HashMap::new(),
        }
    }

    pub fn from_file(path: PathBuf) -> Self {
        Self {
            lines: Lines::Buffered(Box::new(BufReader::new(fs::File::open(&path).unwrap()))),
            interactive: false,
            name: String::from(path.file_name().unwrap().to_str().unwrap()),
            vars: HashMap::new(),
        }
    }

    pub fn interactive() -> Self {
        Self {
            lines: Lines::Buffered(Box::new(BufReader::new(io::stdin()))),
            interactive: true,
            name: String::from(crate_name!()),
            vars: HashMap::new(),
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
