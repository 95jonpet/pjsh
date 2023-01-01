use std::path::Path;

use itertools::Itertools;
use pjsh_core::{Completion, Completions, Context};
use pjsh_eval::interpolate_function_call;

use super::{fs::complete_paths, Replacement};

/// Completes a word based on a prefix.
pub fn complete_in_context(
    prefix: &str,
    words: &[&str],
    word_index: usize,
    context: &Context,
    completions: &Completions,
) -> Option<Vec<Replacement>> {
    if word_index == 0 {
        return None;
    }

    let Some(completion) = completions.get(words[0]) else {
        return None;
    };

    Some(match completion {
        Completion::Constant(words) => complete_words(prefix, words),
        Completion::Directory => complete_paths(prefix, context, Path::is_dir),
        Completion::File => complete_paths(prefix, context, Path::exists),
        Completion::Function(function_name) => {
            let Some(function) = context.get_function(function_name) else {
                return Some(Vec::new());
            };

            let args: Vec<String> = words.iter().map(ToString::to_string).collect();

            let Ok(output) = interpolate_function_call(function, &args, context) else {
                return Some(Vec::new());
            };

            output
                .split_whitespace()
                .sorted()
                .map(|word| Replacement::new(word.to_string()))
                .collect()
        }
    })
}

/// Completes a word based on pre-defined words.
fn complete_words(prefix: &str, words: &[String]) -> Vec<Replacement> {
    let mut completions: Vec<Replacement> = words
        .iter()
        .filter(|word| word.starts_with(prefix))
        .cloned()
        .map(Replacement::new)
        .collect();
    completions.sort_by(|a, b| a.content.cmp(&b.content));
    completions
}
