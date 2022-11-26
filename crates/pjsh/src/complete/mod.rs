use pjsh_core::Context;

mod contextualized_complete;
mod uncontextualized_complete;

/// Completes a word based on a prefix.
pub fn complete(prefix: &str, words: &[&str], word_index: usize, context: &Context) -> Vec<String> {
    let completions = contextualized_complete::complete(prefix, words, word_index, &context);
    if !completions.is_empty() {
        return completions;
    }

    uncontextualized_complete::complete(prefix, words, word_index, context)
}
