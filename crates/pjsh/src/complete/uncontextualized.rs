use std::collections::HashSet;

use is_executable::is_executable;
use itertools::{chain, Itertools};
use pjsh_core::{paths, Context};

use super::{fs::complete_paths, Replacement};

/// Completes a word based on a prefix.
pub fn complete_anything(
    prefix: &str,
    _words: &[&str],
    word_index: usize,
    context: &Context,
) -> Vec<Replacement> {
    // Complete references to things that may be executable if completing the first
    // word, i.e. the program.
    if word_index == 0 {
        let mut replacements: Vec<Replacement> = chain!(
            complete_aliases(prefix, context),
            complete_builtins(prefix, context),
            complete_functions(prefix, context),
            complete_variables(prefix, context),
            complete_programs(prefix, context),
            complete_paths(prefix, context, |path| path.is_dir() || is_executable(path)),
        )
        .unique()
        .collect();

        replacements.sort_by(|a, b| a.content.cmp(&b.content));
        return replacements;
    }

    // Complete paths if starting a new word.
    if prefix.is_empty() {
        return complete_paths(prefix, context, |_| true);
    }

    // Otherwise, complete a generic argument-like word.
    let mut replacements: Vec<Replacement> = chain!(
        complete_variables(prefix, context),
        complete_paths(prefix, context, |_| true),
    )
    .unique()
    .collect();

    replacements.sort_by(|a, b| a.content.cmp(&b.content));
    replacements
}

/// Completes an alias.
fn complete_aliases<'a>(
    prefix: &'a str,
    context: &'a Context,
) -> impl Iterator<Item = Replacement> + 'a {
    context.aliases.iter().filter_map(move |(name, _)| {
        if name.starts_with(prefix) {
            Some(Replacement::new(name.to_string()))
        } else {
            None
        }
    })
}

/// Completes a built-in function name.
fn complete_builtins<'a>(
    prefix: &'a str,
    context: &'a Context,
) -> impl Iterator<Item = Replacement> + 'a {
    context.builtins.iter().filter_map(move |(name, _)| {
        if name.starts_with(prefix) {
            Some(Replacement::new(name.to_string()))
        } else {
            None
        }
    })
}

/// Completes a function name.
fn complete_functions<'a>(
    prefix: &'a str,
    context: &'a Context,
) -> impl Iterator<Item = Replacement> + 'a {
    context
        .get_function_names()
        .into_iter()
        .filter(move |name| name.starts_with(prefix))
        .map(Replacement::new)
}

/// Completes a program name.
fn complete_programs(prefix: &str, context: &Context) -> Vec<Replacement> {
    let mut programs = HashSet::new();
    for dir in paths(context) {
        let Ok(files) = std::fs::read_dir(dir) else {
            continue
        };

        for file in files {
            let Ok(file) = file else {
                continue
            };

            let name = file.file_name().to_string_lossy().to_string();
            if !name.starts_with(prefix) || !is_executable(file.path()) {
                continue;
            }

            programs.insert(name);
        }
    }
    programs.into_iter().map(Replacement::new).collect()
}

/// Completes a variable.
fn complete_variables(prefix: &str, context: &Context) -> Vec<Replacement> {
    let Some(prefix) = prefix.strip_prefix('$') else {
        return Vec::default();
    };

    context
        .get_var_names()
        .iter()
        .filter(|name| name.starts_with(prefix))
        .map(|name| Replacement::new(format!("${name}")))
        .collect()
}

/// Complete well-known prefixes.
pub(crate) fn complete_known_prefix(prefix: &str) -> Option<Vec<Replacement>> {
    match prefix {
        ".." => Some(vec![Replacement::from("../")]),
        _ => None,
    }
}
