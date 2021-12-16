mod alias;
mod glob;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;

use pjsh_ast::Word;
use pjsh_core::Context;

use crate::expand::alias::expand_aliases;
use crate::expand::glob::expand_globs;
use crate::word::interpolate_word;

/// Interpolates and expands words.
///
/// Expands a [`Vec<Word>`] into a [`VecDeque<String>`] by interpolating each word and expanding
/// them:
/// 1. aliases in the current [`Context`].
/// 2. filesystem globs.
pub fn expand(words: Vec<Word>, context: &Context) -> VecDeque<String> {
    let mut words = interpolate_words(words, context); // (text, expandable)
    debug_assert!(!words.is_empty(), "words should not be empty");

    expand_aliases(&mut words, context);
    expand_globs(&mut words, context);

    words.into_iter().map(|(word, _)| word).collect()
}

/// Interpolates words and converts them into strings.
fn interpolate_words(words: Vec<Word>, context: &Context) -> VecDeque<(String, bool)> {
    words
        .into_iter()
        .map(|word| {
            let expandable = matches!(word, Word::Literal(_) | Word::Variable(_));
            let interpolated = interpolate_word(word, context);
            (interpolated, expandable)
        })
        .collect()
}
