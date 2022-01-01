use std::iter::Peekable;

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use super::lexer::Span;

/// Grapheme cluster representing the end of input.
const EOF: &str = "\0";

/// Iterator over a sequence of grapheme clusters.
pub struct Input<'a> {
    /// Position and grapheme cluster representing the end of input. The position is equal to input
    /// length + 1.
    eof: (usize, &'a str),
    /// Peekable iterator over the individual grapheme clusters that make up the input.
    grapheme_clusters: Peekable<GraphemeIndices<'a>>,
}

impl<'a> Input<'a> {
    /// Constructs a new input iterator from some input.
    pub fn new(src: &'a str) -> Self {
        Self {
            eof: (src.len(), EOF),
            grapheme_clusters: src.grapheme_indices(true).peekable(),
        }
    }

    /// Advances the iterator and returns the next value.
    pub fn next(&mut self) -> (usize, &str) {
        self.grapheme_clusters.next().unwrap_or(self.eof)
    }

    /// Returns a reference to the [`next()`] value without advancing the iterator.
    pub fn peek(&mut self) -> &(usize, &str) {
        self.grapheme_clusters.peek().unwrap_or(&self.eof)
    }

    /// Consume and return the next value of this iterator if a condition is true.
    ///
    /// If `func` returns `true` for the next value of this iterator, consume and return it.
    /// Otherwise, return `None`.
    pub fn next_if(&mut self, func: impl FnOnce(&str) -> bool) -> Option<(usize, &str)> {
        self.grapheme_clusters.next_if(|head| func(head.1))
    }

    /// Consume and return the next value of this iterator if `ch` is equal to the next grapheme
    /// cluster.
    pub fn next_if_eq(&mut self, ch: &str) -> Option<(usize, &str)> {
        self.grapheme_clusters.next_if(|head| head.1 == ch)
    }

    /// Takes the next `chars` (grapheme clusters of input) from the iterator and return their span
    /// if they match. Otherwise, return `None`.
    pub fn take_if_eq(&mut self, chars: &[&str]) -> Option<Span> {
        let mut original_iterator = self.grapheme_clusters.clone();

        for ch in chars {
            if self.next_if_eq(ch).is_none() {
                self.grapheme_clusters = original_iterator;
                return None;
            }
        }

        Some(Span::new(
            original_iterator.peek().unwrap_or(&self.eof).0,
            self.grapheme_clusters.peek().unwrap_or(&self.eof).0,
        ))
    }

    /// Returns a references to the `n` [`next()`] values without advancing the iterator.
    pub fn peek_n(&self, n: usize) -> Vec<&str> {
        let mut input = self.grapheme_clusters.clone();

        let mut peeked = Vec::new();
        for _ in 0..n {
            peeked.push(input.next().unwrap_or(self.eof).1);
        }

        peeked
    }

    /// Returns a accumulated span and string for the [`next()`] values while a `predicate` returns
    /// `true`.
    pub fn eat_while(&mut self, mut predicate: impl FnMut(&str) -> bool) -> (Span, String) {
        let mut content = String::new();
        let start = self.peek().0;
        let mut end = start;

        while let Some(head) = self.next_if(|ch| predicate(ch)) {
            content.push_str(head.1);
            end = head.0 + 1;
        }

        (Span::new(start, end), content)
    }
}

/// Returns `true` if a unicode grapheme cluster should be considered a newline.
pub fn is_newline(grapheme_cluster: &str) -> bool {
    matches!(
        grapheme_cluster,
        "\u{000A}"   // \n
        | "\u{000B}" // vertical tab
        | "\u{000C}" // form feed
        | "\u{000D}" // \r
        | "\u{0085}" // next line
        | "\u{2028}" // line separator
        | "\u{2029}" // paragraph separator
        | "\r\n"
    )
}

/// Returns `true` if a character is allowed in a variable name.
pub fn is_variable_char(c: &str) -> bool {
    c == "_" || c.chars().all(char::is_alphanumeric)
}

/// Returns `true` if a unicode grapheme cluster should be considered whitespace.
pub fn is_whitespace(grapheme_cluster: &str) -> bool {
    matches!(
        grapheme_cluster,
        // ASCII
        "\u{0009}"   // \t
        | "\u{000A}" // \n
        | "\u{000B}" // vertical tab
        | "\u{000C}" // form feed
        | "\u{000D}" // \r
        | "\u{0020}" // space

        // NEXT LINE from latin1
        | "\u{0085}"

        // Bidi markers
        | "\u{200E}" // LEFT-TO-RIGHT MARK
        | "\u{200F}" // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | "\u{2028}" // LINE SEPARATOR
        | "\u{2029}" // PARAGRAPH SEPARATOR
    )
}

/// Returns `true` if a unicode grapheme cluster can belong to a literal.
pub fn is_literal(grapheme_cluster: &str) -> bool {
    if is_whitespace(grapheme_cluster) {
        return false;
    }

    // Reserved non-literal characters.
    if matches!(
        grapheme_cluster,
        "(" | ")" | "{" | "}" | "[" | "]" | "<" | ">"
    ) {
        return false;
    }

    true
}
