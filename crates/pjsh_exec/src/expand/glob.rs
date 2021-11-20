use std::{collections::VecDeque, path::PathBuf};

use pjsh_core::Context;

/// Expands glob patterns matching file names.
pub fn expand_globs(words: &mut VecDeque<String>, context: &Context) {
    expand_tilde(words, context);
    expand_asterisk(words, context);
}

/// Expands tilde (`~`) to the value of `$HOME` if it is the first character of a word.
/// Any other tilde characters are left as is.
fn expand_tilde(words: &mut VecDeque<String>, context: &Context) {
    for word in words {
        if word.starts_with('~') {
            let home = context
                .scope
                .get_env("HOME")
                .unwrap_or_else(|| String::from("/"));

            // Replace the word.
            *word = word.replacen('~', &home, 1);
        }
    }
}

/// Replace any asterisk (`*`) with the name of files and folders.
fn expand_asterisk(words: &mut VecDeque<String>, _context: &Context) {
    let mut new_words = VecDeque::<String>::with_capacity(words.capacity());
    while let Some(word) = words.pop_front() {
        if let Some(index) = word.find('*') {
            let head = &word[0..index];
            let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            path.push(head);

            // Cannot expand glob, keep the asterisk.
            if !path.exists() {
                new_words.push_back(word);
                continue;
            }

            let mut globbed = Vec::new();
            for entry in path.read_dir().unwrap() {
                let file_name = entry.unwrap().file_name().to_string_lossy().to_string();

                if file_name.starts_with('.') {
                    continue;
                }

                let mut file = word.clone();
                file.replace_range(index..index + 1, &file_name);
                globbed.push(file);
            }
            globbed.sort();
            for word in globbed {
                new_words.push_back(word);
            }
        } else {
            new_words.push_back(word);
        }
    }

    *words = new_words;
}
