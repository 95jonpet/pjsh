mod alias;
mod glob;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_ast::Word;
use pjsh_core::Context;

use crate::expand::alias::expand_aliases;
use crate::expand::glob::expand_globs;
use crate::word::interpolate_word;
use crate::Executor;

/// Interpolates and expands words.
///
/// Expands a [`Vec<Word>`] into a [`VecDeque<String>`] by interpolating each word and expanding
/// them:
/// 1. aliases in the current [`Context`].
/// 2. filesystem globs.
pub fn expand(
    words: Vec<Word>,
    context: Arc<Mutex<Context>>,
    executor: &Executor,
) -> VecDeque<String> {
    let mut words = interpolate_words(words, Arc::clone(&context), executor); // (text, expandable)
    debug_assert!(!words.is_empty(), "words should not be empty");

    expand_aliases(&mut words, &context.lock());
    expand_globs(&mut words, &context.lock());

    words.into_iter().map(|(word, _)| word).collect()
}

/// Interpolates and expands a single word.
///
/// Returns `None` if the given word expands to multiple words.
pub fn expand_single(
    word: Word,
    context: Arc<Mutex<Context>>,
    executor: &Executor,
) -> Option<String> {
    let mut expanded = expand(vec![word], context, executor);

    if expanded.len() != 1 {
        return None;
    }

    expanded.pop_front()
}

/// Interpolates words and converts them into strings.
fn interpolate_words(
    words: Vec<Word>,
    context: Arc<Mutex<Context>>,
    executor: &Executor,
) -> VecDeque<(String, bool)> {
    words
        .into_iter()
        .map(|word| {
            let expandable = matches!(word, Word::Literal(_) | Word::Variable(_));
            let interpolated = interpolate_word(executor, word, Arc::clone(&context));
            (interpolated, expandable)
        })
        .collect()
}
