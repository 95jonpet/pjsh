use crate::Replacement;

/// Complete well-known prefixes.
pub(crate) fn complete_known_prefix(prefix: &str) -> Option<Vec<Replacement>> {
    match prefix {
        ".." => Some(vec![Replacement::from("../".to_string())]),
        _ => None,
    }
}
