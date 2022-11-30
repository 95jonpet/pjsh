use is_executable::is_executable;
use itertools::chain;
use pjsh_core::Context;

use super::fs::complete_paths;

/// Completes a word based on a prefix.
pub fn complete(
    prefix: &str,
    _words: &[&str],
    word_index: usize,
    context: &Context,
) -> Vec<String> {
    // Complete references to things that may be executable if completing the first
    // word, i.e. the program.
    if word_index == 0 {
        return chain!(
            complete_aliases(prefix, context),
            complete_builtins(prefix, context),
            complete_functions(prefix, context),
            complete_variables(prefix, context),
            complete_paths(prefix, context, |path| is_executable(path)),
        )
        .collect();
    }

    // Complete paths if starting a new word.
    if prefix.is_empty() {
        return complete_paths(prefix, context, |_| true);
    }

    // Otherwise, complete a generic argument-like word.
    chain!(
        complete_variables(prefix, context),
        complete_paths(prefix, context, |_| true),
    )
    .collect()
}

/// Completes an alias.
fn complete_aliases(prefix: &str, context: &Context) -> Vec<String> {
    context
        .aliases
        .iter()
        .map(|(name, _)| name)
        .filter(|name| name.starts_with(prefix))
        .cloned()
        .collect()
}

/// Completes a built-in function name.
fn complete_builtins(prefix: &str, context: &Context) -> Vec<String> {
    context
        .builtins
        .iter()
        .map(|(name, _)| name)
        .filter(|name| name.starts_with(prefix))
        .cloned()
        .collect()
}

/// Completes a function name.
fn complete_functions(prefix: &str, context: &Context) -> Vec<String> {
    context
        .get_function_names()
        .iter()
        .filter(|name| name.starts_with(prefix))
        .cloned()
        .collect()
}

/// Completes a variable.
fn complete_variables(prefix: &str, context: &Context) -> Vec<String> {
    let Some(prefix) = prefix.strip_prefix('$') else {
        return Vec::default();
    };

    context
        .get_var_names()
        .iter()
        .filter(|name| name.starts_with(prefix))
        .map(|name| format!("${name}"))
        .collect()
}
