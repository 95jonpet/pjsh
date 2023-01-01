use pjsh_core::{Completions, Context};

use self::{
    contextualized::complete_in_context,
    uncontextualized::{complete_anything, complete_known_prefix},
};

mod contextualized;
mod fs;
mod uncontextualized;

/// A completed word.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Replacement {
    /// Raw completed value to replace.
    pub(crate) content: String,

    /// Completed value to display.
    pub(crate) display: Option<String>,
}

impl Replacement {
    /// Constructs a new completion.
    pub fn new(content: String) -> Self {
        Self {
            content,
            display: None,
        }
    }

    /// Constructs a new completion with a customized display value.
    pub fn customized(content: String, display: String) -> Self {
        Self {
            content,
            display: Some(display),
        }
    }
}

impl From<&str> for Replacement {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// Completes a word based on a prefix.
pub fn complete(
    prefix: &str,
    words: &[&str],
    word_index: usize,
    context: &Context,
    completions: &Completions,
) -> Vec<Replacement> {
    complete_known_prefix(prefix)
        .or_else(|| complete_in_context(prefix, words, word_index, context, completions))
        .or_else(|| Some(complete_anything(prefix, words, word_index, context)))
        .unwrap_or_default()
}
