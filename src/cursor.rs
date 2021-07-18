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
