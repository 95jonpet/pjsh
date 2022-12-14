mod fs;

#[cfg(test)]
mod tests;

pub use fs::{path_to_string, resolve_path};

use crate::{env::context::Value, Context};

/// Returns a single word variable from a context.
pub fn word_var<'a>(context: &'a Context, name: &str) -> Option<&'a str> {
    if let Some(Value::Word(word)) = context.get_var(name) {
        return Some(word);
    }

    None
}
