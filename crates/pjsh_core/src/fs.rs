use std::path::PathBuf;

use crate::{
    utils::{resolve_path, word_var},
    Context,
};

/// Find a program by searching for its name in the paths present in `$PATH`.
///
/// Optionally, extensions present in the semicolon-separated `$PATHEXT` are used when searching.
/// Note that `$PATHEXT` is typically only present on Windows systems. If the environment variable
/// is undefined, only the name is matched.
///
/// Also note that file system case-insensitivity may be in effect.
pub fn find_in_path(name: &str, context: &Context) -> Option<PathBuf> {
    // Match an exact program path.
    if name.contains('/') {
        return Some(resolve_path(context, name));
    }

    // Define all possible file extensions that can be matched implicitly.
    let mut extensions = vec![String::new()]; // Empty string = no file extension.
    if let Some(ext_env) = word_var(context, "PATHEXT") {
        extensions.extend(ext_env.split(';').map(str::to_owned));
    }

    // Define all possible paths using paths in PATH combined with all possible extensions.
    let paths = paths(context);
    let possible_paths = paths.iter().flat_map(|path| {
        extensions.iter().map(|extension| {
            let mut path = path.clone();
            path.push(name.to_owned() + extension);
            path
        })
    });

    // Search through all possible paths for a matching file.
    possible_paths
        .into_iter()
        .find(|path| path.exists())
        .map(|path| path.canonicalize().unwrap_or(path))
}

/// Returns a list of all paths in `$PATH` separated by ':' on Unix systems, and
/// by ';' on Windows.
pub fn paths(context: &Context) -> Vec<PathBuf> {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let path_string = word_var(context, "PATH").unwrap_or_default();
    path_string.split(separator).map(PathBuf::from).collect()
}

#[cfg(test)]
mod tests {
    use crate::Value;

    use super::*;

    #[test]
    fn it_splits_paths() {
        let separator = if cfg!(windows) { ';' } else { ':' };
        let mut context = Context::default();
        context.set_var(
            "PATH".into(),
            Value::Word(format!("/tmp/a{separator}/var/tmp/b")),
        );
        assert_eq!(
            paths(&context),
            vec![PathBuf::from("/tmp/a"), PathBuf::from("/var/tmp/b")]
        );
    }
}
