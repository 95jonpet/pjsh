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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn it_can_get_the_next_line_from_a_line_buffer() {
        let input = vec!["first\n", "second\n"];
        let io_cursor = io::Cursor::new(input.join(""));
        let mut lines = InputLines::Buffered(Box::new(io_cursor));
        assert_eq!(lines.next(), Some(input[0].to_string()), "first line");
        assert_eq!(lines.next(), Some(input[1].to_string()), "second line");
        assert_eq!(lines.next(), None, "no more lines");
    }

    #[test]
    fn it_can_get_the_next_line_from_single_line_input() {
        let line = String::from("input\n");
        let mut lines = InputLines::Single(Some(line.clone()));
        assert_eq!(lines.next(), Some(line), "return the complete input");
        assert_eq!(lines.next(), None, "no more lines");
    }
}
