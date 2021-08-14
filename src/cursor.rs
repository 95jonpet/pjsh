use std::{
    cell::RefCell,
    io::{self, Write},
    iter::Peekable,
    rc::Rc,
    vec::IntoIter,
};

use crate::{input::InputLines, options::Options};

/// Peekable iterator over a char stream.
pub struct Cursor {
    input: Peekable<InputLines>,
    interactive: bool,
    line: Peekable<IntoIter<char>>,
    line_buffer: String,
    line_offset: usize,
    line_number: usize,
    options: Rc<RefCell<Options>>,
}

/// Character representing the end of file/input.
pub(crate) const EOF_CHAR: char = '\0';

pub(crate) static PS1: &str = "$ ";
pub(crate) static PS2: &str = "> ";

impl Cursor {
    /// Creates a new cursor for iterating over a char stream.
    pub fn new(input: InputLines, interactive: bool, options: Rc<RefCell<Options>>) -> Self {
        Self {
            input: input.peekable(),
            interactive,
            line: Vec::new().into_iter().peekable(),
            line_buffer: String::new(),
            line_offset: 0,
            line_number: 0,
            options,
        }
    }

    /// Returns the next [`char`] from the input stream without consuming it.
    /// If the input has been fully consumed, [`EOF_CHAR`] is returned.
    pub fn peek(&mut self) -> &char {
        self.line.peek().unwrap_or(&EOF_CHAR)
    }

    /// Returns the next [`char`] from the input stream and consumes it.
    /// If the current line ends, the iterator moves to the next line.
    /// If the input has been fully consumed, [`EOF_CHAR`] is returned.
    pub fn next(&mut self) -> char {
        if let Some(ch) = self.line.next() {
            self.line_offset += 1;
            return ch;
        }

        EOF_CHAR
    }

    /// Skips a specific [`char`].
    /// Panics if the next character is not the expected character.
    pub fn skip(&mut self, ch: char) {
        if &ch != self.peek() {
            panic!("expected character '{}' but saw '{}'", ch, self.peek(),);
        }

        self.next();
    }

    /// Returns `true` if the cursor is interactive.
    /// In interactive mode, each line should be parsed and executed immediately.
    #[inline]
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    /// Moves the iterator to the next line of input.
    pub(crate) fn advance_line(&mut self, prompt: &str) {
        if self.is_interactive() {
            print!("{}", prompt);
            io::stdout().flush().unwrap();
        }

        if let Some(line) = self.input.next() {
            // Print read input to stderr if requested.
            if self.options.borrow().print_input {
                eprint!("{}", line); // Is expected to contain a newline.
            }

            self.line = line.chars().collect::<Vec<_>>().into_iter().peekable();
            self.line_buffer = line;
            self.line_number += 1;
            self.line_offset = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_peeks_the_next_char() {
        assert_eq!(cursor("12").peek(), &'1', "return the first character");
        assert_eq!(cursor("").peek(), &EOF_CHAR, "no more input, return EOF");
    }

    #[test]
    fn it_returns_the_next_char() {
        let mut cursor = cursor("12");
        assert_eq!(cursor.next(), '1', "return the first character");
        assert_eq!(cursor.next(), '2', "return the second character");
        assert_eq!(cursor.next(), EOF_CHAR, "no more input, return EOF");
        assert_eq!(cursor.next(), EOF_CHAR, "don't panic on further reads");
    }

    #[test]
    fn it_advances_lines() {
        let mut cursor = multiline_cursor(vec!["1\n", "2\n"]);
        assert_eq!(cursor.peek(), &'1', "first character of the first line");
        cursor.advance_line("");
        assert_eq!(cursor.peek(), &'2', "first character of the second line");
    }

    #[test]
    fn it_knows_if_it_is_interactive() {
        let options = Rc::new(RefCell::new(Options::default()));
        let non_interactive = Cursor::new(InputLines::Single(None), false, options.clone());
        let interactive = Cursor::new(InputLines::Single(None), true, options);

        assert_eq!(non_interactive.is_interactive(), false);
        assert_eq!(interactive.is_interactive(), true);
    }

    #[test]
    fn it_can_skip_characters() {
        let mut cursor = cursor("12");
        cursor.skip('1');
        assert_eq!(cursor.peek(), &'2');
    }

    #[test]
    #[should_panic(expected = "expected character '2' but saw '1'")]
    fn it_panics_when_trying_to_skip_invalid_characters() {
        let mut cursor = cursor("12");
        cursor.skip('2');
    }

    fn multiline_cursor(lines: Vec<&str>) -> Cursor {
        let io_cursor = io::Cursor::new(lines.join(""));
        let mut cursor = Cursor::new(
            InputLines::Buffered(Box::new(io_cursor)),
            false,
            Rc::new(RefCell::new(Options::default())),
        );
        cursor.advance_line(""); // Force the first line of input to be read.
        cursor
    }

    fn cursor(input: &str) -> Cursor {
        let mut cursor = Cursor::new(
            InputLines::Single(Some(String::from(input))),
            false,
            Rc::new(RefCell::new(Options::default())),
        );
        cursor.advance_line(""); // Force the first line of input to be read.
        cursor
    }
}
