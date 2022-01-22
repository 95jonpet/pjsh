use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::Context;

/// Converts a path to a string.
///
/// Non-unicode characters are replaced by '?' in the returned string.
pub fn path_to_string<P: AsRef<Path>>(path: &P) -> String {
    path.as_ref()
        .to_string_lossy()
        .trim_start_matches(r#"\\?\"#)
        .to_string()
}

/// Resolves a path given a [`Context`].
///
/// Relative paths are resolved from the current working directory as determined by `$PWD`. Path
/// resolution is not guaranteed when `$PWD` is not set.
///
/// Returns a canonicalized (absolute) path.
pub fn resolve_path<P: AsRef<OsStr>>(context: &Context, path: P) -> PathBuf {
    let mut resolved_path = context
        .scope
        .get_env("PWD")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"));
    resolved_path.push(path.as_ref());

    // Attempt to canonicalize the path into an absolute path.
    resolved_path.canonicalize().unwrap_or(resolved_path)
}
