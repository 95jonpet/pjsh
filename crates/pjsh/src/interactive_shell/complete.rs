use std::{fs, path::Path};

use itertools::chain;
use pjsh_core::{
    utils::{path_to_string, resolve_path},
    Context,
};

/// Completes a word based on a prefix.
pub fn complete(
    prefix: &str,
    _words: &[&str],
    word_index: usize,
    context: &Context,
) -> Vec<String> {
    // Complete an argument.
    if word_index != 0 {
        return chain!(
            complete_variables(prefix, context),
            complete_paths(prefix, context),
        )
        .collect();
    }

    // Complete a program/function name.
    chain!(
        complete_aliases(prefix, context),
        complete_builtins(prefix, context),
        complete_functions(prefix, context),
        complete_variables(prefix, context),
        complete_paths(prefix, context),
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

/// Completes a path.
fn complete_paths(prefix: &str, context: &Context) -> Vec<String> {
    if let Some((dir, file_prefix)) = prefix.rsplit_once('/') {
        let dir_path = resolve_path(context, dir);
        let Ok(files) = fs::read_dir(dir_path) else {
            return Vec::default();
        };

        return files
            .into_iter()
            .filter_map(|file| {
                let file = filtered_file_name(file.ok()?.path(), file_prefix)?;
                Some(format!("{dir}/{}", file))
            })
            .collect();
    }

    let Ok(Ok(files)) = std::env::current_dir().map(fs::read_dir) else {
        return Vec::default();
    };

    files
        .into_iter()
        .filter_map(|file| filtered_file_name(file.ok()?.path(), prefix))
        .collect()
}

/// Returns a filtered file name.
fn filtered_file_name<P: AsRef<Path>>(path: P, name_prefix: &str) -> Option<String> {
    let path_str = path_to_string(path);
    let (_, file_str) = path_str.rsplit_once('/')?;

    if !file_str.starts_with(name_prefix) {
        return None;
    }

    Some(file_str.to_owned())
}
