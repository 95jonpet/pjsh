use rustyline::completion::{Candidate, Completer, Pair};

use crate::completion::Complete;

const WORD_BOUNDARY: [char; 11] = [
    // '(',
    // ')',
    // '{',
    // '}',
    // '[',
    // ']',
    '\u{0009}', // \t
    '\u{000A}', // \n
    '\u{000B}', // vertical tab
    '\u{000C}', // form feed
    '\u{000D}', // \r
    '\u{0020}', // space
    // NEXT LINE from latin1
    '\u{0085}', // Bidi markers
    '\u{200E}', // LEFT-TO-RIGHT MARK
    '\u{200F}', // RIGHT-TO-LEFT MARK
    // Dedicated whitespace characters from Unicode
    '\u{2028}', // LINE SEPARATOR
    '\u{2029}', // PARAGRAPH SEPARATOR
];

pub struct CombinationCompleter {
    completers: Vec<Box<dyn Complete>>,
}

impl CombinationCompleter {
    pub fn new(completers: Vec<Box<dyn Complete>>) -> Self {
        Self { completers }
    }
}

impl Completer for CombinationCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let start_index = line
            .char_indices()
            .rev()
            .skip(line.len() - pos)
            .find(|(_, ch)| WORD_BOUNDARY.contains(ch))
            .map(|(index, _)| index + 1)
            .unwrap_or(0);

        let current_word = unquote(&line[start_index..pos]);

        let mut completions = Vec::new();
        for completer in &self.completers {
            for completion in completer.complete(current_word) {
                let quoted = quote(completion);
                completions.push(Pair {
                    display: quoted.clone(),
                    replacement: quoted,
                });
            }
        }

        completions.sort_by(|a, b| a.display().cmp(b.display()));

        Ok((start_index, completions))
    }
}

fn quote(path: String) -> String {
    if !path.contains(char::is_whitespace) {
        return path;
    }

    format!("`{}`", path)
}

fn unquote(path: &str) -> &str {
    let should_trim = |ch: char| ch.is_whitespace() || ch == '`';
    let mut path = path.trim_end_matches(should_trim);
    path = path.trim_start_matches(should_trim);
    path
}
