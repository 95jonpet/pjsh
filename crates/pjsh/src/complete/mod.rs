use pjsh_core::{Completions, Context};

mod contextualized_complete;
mod fs;
mod uncontextualized_complete;

/// Completes a word based on a prefix.
pub fn complete(
    prefix: &str,
    words: &[&str],
    word_index: usize,
    context: &Context,
    completions: &Completions,
) -> Vec<String> {
    contextualized_complete::complete(prefix, words, word_index, context, completions)
        .unwrap_or_else(|| uncontextualized_complete::complete(prefix, words, word_index, context))
}
