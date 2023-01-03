use std::collections::HashMap;

use pjsh_core::Context;

use crate::{
    completions::Completion, input::separate_input, known_prefixes::complete_known_prefix,
    registered_completions::complete_registered, uncontextualized_completions::complete_anything,
    LineCompletion, Replacement,
};

#[derive(Debug, Default)]
pub struct Completer {
    completions: HashMap<String, Completion>,
}

impl Completer {
    pub fn complete_line(&self, line: &str, pos: usize, context: &Context) -> LineCompletion {
        let mut words = separate_input(line);

        // The current position may be inside whitespace following the final word.
        // If this is the case, completions should be provided for a new word with an
        // empty prefix. They should, however, not be provided for the first word.
        if pos > words.last().map_or(usize::MAX, |(_, _, end)| *end) {
            words.push(("", pos, pos));
        }

        let Some(word_index) = words
            .iter()
            .position(|(_, start, end)| pos >= *start && pos <= *end) else {
                // No input to complete.
                return LineCompletion::new(pos, Vec::new());
            };

        let word = words[word_index];
        let prefix = &word.0[..(pos - word.1)];

        let words: Vec<&str> = words.into_iter().map(|(word, _, _)| word).collect();

        let completions = self.complete_word(prefix, &words, word_index, context);
        LineCompletion::new(word.1, completions)
    }

    /// Registers a completion for a program.
    pub fn register_completion(&mut self, program: String, completion: Completion) {
        self.completions.insert(program, completion);
    }

    /// Completes a word based on a prefix.
    fn complete_word(
        &self,
        prefix: &str,
        words: &[&str],
        word_index: usize,
        context: &Context,
    ) -> Vec<Replacement> {
        complete_known_prefix(prefix)
            .or_else(|| complete_registered(prefix, words, word_index, context, &self.completions))
            .unwrap_or_else(|| complete_anything(prefix, words, word_index, context))
    }
}
