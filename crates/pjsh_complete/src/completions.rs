#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Replacement {
    /// The suggested content.
    pub content: String,

    /// The visual representation of the suggested content.
    pub display: String,
}

impl Replacement {
    pub fn new(content: String, display: String) -> Self {
        Self { content, display }
    }
}

impl From<String> for Replacement {
    fn from(value: String) -> Self {
        Self::new(value.clone(), value)
    }
}

impl From<&str> for Replacement {
    fn from(value: &str) -> Self {
        Self::new(value.to_string(), value.to_string())
    }
}

pub struct LineCompletion {
    pub line_pos: usize,
    pub replacements: Vec<Replacement>,
}

impl LineCompletion {
    pub(crate) fn new(line_pos: usize, replacements: Vec<Replacement>) -> Self {
        Self {
            line_pos,
            replacements,
        }
    }
}

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
