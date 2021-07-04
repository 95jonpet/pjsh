use std::{
    io::{self, Write},
    iter::Peekable,
    vec::IntoIter,
};

use crate::input::InputLines;

/// Peekable iterator over a char stream.
pub struct Cursor {
    input: Peekable<InputLines>,
    interactive: bool,
    line: Peekable<IntoIter<char>>,
    line_buffer: String,
    line_offset: usize,
    line_number: usize,
}

/// Character representing the end of file/input.
pub(crate) const EOF_CHAR: char = '\0';

impl Cursor {
    pub fn new(input: InputLines, interactive: bool) -> Self {
        Self {
            input: input.peekable(),
            interactive,
            line: Vec::new().into_iter().peekable(),
            line_buffer: String::new(),
            line_offset: 0,
            line_number: 0,
        }
    }

    /// Returns the next [`char`] from the input stream without consuming it.
    /// If the input has been fully consumed, [`EOF_CHAR`] is returned.
    pub fn peek(&mut self) -> &char {
        if self.line_offset >= self.line_buffer.len() {
            self.advance_line();
        }

        self.line.peek().unwrap_or(&EOF_CHAR)
    }

    /// Returns the next [`char`] from the input stream and consumes it.
    /// If the current line ends, the iterator moves to the next line.
    /// If the input has been fully consumed, [`EOF_CHAR`] is returned.
    pub fn next(&mut self) -> char {
        if self.line_offset >= self.line_buffer.len() {
            self.advance_line();
        }

        match self.line.next() {
            Some(ch) => {
                self.line_offset += 1;
                ch
            }
            None => EOF_CHAR,
        }
    }

    /// Returns a [`String`] of all characters until a predicate `P` holds.
    /// All returned [`char`] elements inside the string are consumed.
    pub fn read_until<P>(&mut self, predicate: P) -> String
    where
        P: Fn(&char) -> bool,
    {
        let mut result = String::new();
        loop {
            match self.peek() {
                ch if !predicate(ch) && ch != &EOF_CHAR => {
                    let c = self.next();
                    result.push(c);
                }
                _ => break,
            }
        }
        result
    }

    /// Returns `true` if the cursor is interactive.
    /// In interactive mode, each line should be parsed and executed immediately.
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    /// Moves the iterator to the next line of input.
    fn advance_line(&mut self) {
        if self.is_interactive() {
            print!("{}", "$ ");
            io::stdout().flush().unwrap();
        }

        match self.input.next() {
            Some(line) => {
                self.line = line.chars().collect::<Vec<_>>().into_iter().peekable();
                self.line_buffer = line;
                self.line_number += 1;
                self.line_offset = 0;
            }
            None => (),
        }
    }
}
