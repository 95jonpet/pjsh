use std::path::Path;

use itertools::Itertools;
use pjsh_core::{Completion, Completions, Context};
use pjsh_eval::interpolate_function_call;

use super::fs::complete_paths;

/// Completes a word based on a prefix.
pub fn complete(
    prefix: &str,
    words: &[&str],
    word_index: usize,
    context: &Context,
    completions: &Completions,
) -> Option<Vec<String>> {
    if word_index == 0 {
        return None;
    }

    let Some(completion) = completions.get(words[0]) else {
        return None;
    };

    Some(match completion {
        Completion::Constant(words) => complete_words(prefix, words),
        Completion::Directory => complete_paths(prefix, context, Path::is_dir),
        Completion::File => complete_paths(prefix, context, Path::is_file),
        Completion::Function(function_name) => {
            let Some(function) = context.get_function(function_name) else {
                return Some(Vec::new());
            };

            let Ok(output) = interpolate_function_call(function, &[function_name.to_owned()], context) else {
                return Some(Vec::new());
            };

            output
                .split_whitespace()
                .sorted()
                .map(|word| word.to_string())
                .collect()
        }
    })
}

/// Completes a word based on pre-defined words.
fn complete_words(prefix: &str, words: &[String]) -> Vec<String> {
    let mut completions: Vec<String> = words
        .iter()
        .filter(|word| word.starts_with(prefix))
        .cloned()
        .collect();
    completions.sort();
    completions
}
