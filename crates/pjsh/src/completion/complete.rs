/// Attempts to complete input by making suggestions.
pub trait Complete {
    /// Completes some input.
    ///
    /// Returns a [`Vec<String>`] of possible completions.
    fn complete(&self, input: &str) -> Vec<String>;
}
