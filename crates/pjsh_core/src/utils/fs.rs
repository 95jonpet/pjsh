use std::path::Path;

/// Converts a path to a string.
///
/// Non-unicode characters are replaced by '?' in the returned string.
pub fn path_to_string<P: AsRef<Path>>(path: &P) -> String {
    path.as_ref()
        .to_string_lossy()
        .trim_start_matches(r#"\\?\"#)
        .to_string()
}
