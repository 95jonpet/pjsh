use std::collections::HashMap;

/// A defined shell completion.
#[derive(Debug)]
pub enum Completion {
    /// Complete using a pre-defined list of words.
    Constant(Vec<String>),

    /// Complete a directory path.
    Directory,

    /// Complete a file path.
    File,

    /// Function to execute in order to retrieve completions.
    Function(String),
}

/// A collection of shell completions.
#[derive(Debug, Default)]
pub struct Completions {
    /// Shell completions keyed by their program name.
    completions: HashMap<String, Completion>,
}

impl Completions {
    /// Inserts a completion for a name.
    pub fn insert(&mut self, name: String, completion: Completion) {
        self.completions.insert(name, completion);
    }

    /// Returns a reference to the completion corresponding to the name.
    pub fn get(&self, name: &str) -> Option<&Completion> {
        self.completions.get(name)
    }
}
