use std::collections::{HashSet, VecDeque};

use pjsh_core::Context;

/// Expands alises in a list of words.
///
/// Repeatedly replaces the first word with its aliased value.
///
/// The same alias is never handled more than once. Encountering the same word twice terminates the
/// alias expansion.
///
/// Furthermore, alias expansion is stopped when encountering an alias value ending with a
/// whitespace.
pub fn expand_aliases(words: &mut VecDeque<(String, bool)>, context: &Context) {
    let mut seen_aliases = HashSet::new();
    while let Some((head, true)) = words.pop_front() {
        // Don't use the same alias more than once.
        if seen_aliases.contains(&head) {
            words.push_front((head, true));
            break;
        }

        // Alias - expand to multiple words.
        if let Some(alias) = context.aliases.get(&head) {
            for word in alias.trim_end().split(char::is_whitespace).rev() {
                words.push_front((word.to_string(), true));
            }

            // Stop alias handling if the alias ends with whitespace.
            if alias.ends_with(char::is_whitespace) {
                break;
            }
        } else {
            // Non-alias - keep as-is.
            words.push_front((head, true));
            break;
        }

        seen_aliases.insert(head);
    }
}
