use std::path::PathBuf;

use crate::Context;

/// Find a program by searching for its name in the paths present in `$PATH`.
///
/// Optionally, extensions present in the semicolon-separated `$PATHEXT` are used when searching.
/// Note that `$PATHEXT` is typically only present on Windows systems. If the environment variable
/// is undefined, only the name is matched.
///
/// Also note that file system case-insensitivity may be in effect.
pub fn find_in_path(name: &str, context: &Context) -> Option<PathBuf> {
    // Define all possible file extensions that can be matched.
    let mut extensions = vec![String::new()]; // Empty string = no file extension.
    if let Some(ext_env) = context.scope.get_env("PATHEXT") {
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

    for path in possible_paths {
        if path.exists() {
            return Some(path.canonicalize().unwrap_or(path));
        }
    }

    None
}

/// Returns a list of all paths in `$PATH` separated by ':' on Unix systems, and by ';' on Windows.
fn paths(context: &Context) -> Vec<PathBuf> {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let path_string = context.scope.get_env("PATH").unwrap_or_default();
    path_string.split(separator).map(PathBuf::from).collect()
}
