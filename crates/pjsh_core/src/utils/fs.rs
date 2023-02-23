use std::path::{Path, PathBuf};

use crate::Context;

use super::word_var;

/// Returns a canonical path.
macro_rules! canonical_path {
    ( $base:expr $(, $part:expr)* ) => {
        {
            #[allow(unused_mut)]
            let mut temp_path = PathBuf::from($base);
            $(
                temp_path.push($part);
            )*
            temp_path.canonicalize().unwrap_or(temp_path)
        }
    };
}

/// Converts a path to a string.
///
/// Non-unicode characters are replaced by '?' in the returned string.
pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .to_string_lossy()
        .trim_start_matches(r#"\\?\"#)
        .to_string()
}

/// Resolves a path given a [`Context`].
///
/// Relative paths are resolved from the current working directory as determined by `$PWD`.
/// Path resolution is not guaranteed when `$PWD` and/or `$HOME` are not set within the context.
///
/// Returns a canonicalized (absolute) path.
pub fn resolve_path<P: AsRef<Path>>(context: &Context, path: P) -> PathBuf {
    let path = path.as_ref();

    if path.is_absolute() {
        return canonical_path!(path);
    }

    if path == Path::new("~") {
        return canonical_path!(word_var(context, "HOME").unwrap_or("~"));
    }

    if let Ok(path) = path.strip_prefix("~/") {
        return canonical_path!(word_var(context, "HOME").unwrap_or("~/"), path);
    }

    canonical_path!(word_var(context, "PWD").unwrap_or("/"), path)
}
