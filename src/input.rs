use std::io::BufRead;

/// Iterator over a sequence of [`String`] lines.
pub enum InputLines {
    /// Iterator for a [`BufRead`] buffer.
    Buffered(Box<dyn BufRead>),

    /// Iterator for a single line of input.
    Single(Option<String>),
}

impl Iterator for InputLines {
    type Item = String;

    /// Returns the next line of input.
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
