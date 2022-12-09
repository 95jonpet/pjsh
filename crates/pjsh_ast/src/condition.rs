use crate::Word;

/// A command represents a boolean condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    // Path-related conditions.
    /// True if the given word can be resolved to an existing directory.
    ///
    /// Typically `[[ is-dir word ]]` or `[[ -d word ]]`.
    IsDirectory(Word),

    /// True if the given word can be resolved to an existing file.
    ///
    /// Typically `[[ is-file word ]]` or `[[ -f word ]]`.
    IsFile(Word),

    /// True if the given word can be resolved to an existing file or directory.
    ///
    /// Typically `[[ is-path word ]]` or `[[ -e word ]]`.
    IsPath(Word),

    // Word-related conditions.
    /// True if the given word is empty.
    ///
    /// Typically `[[ -z word ]]`.
    Empty(Word),

    /// True if the given word is not empty.
    ///
    /// Typically `[[ word ]]` or `[[ -n word ]]`.
    NotEmpty(Word),

    // Comparisons.
    /// True if the two given words are considered equal.
    ///
    /// Typically `[[ a == b ]]`.
    Eq(Word, Word),

    /// True if the two given words are not considered equal.
    ///
    /// Typically `[[ a != b ]]`.
    Ne(Word, Word),

    // Misc.
    /// The inverse of another condition.
    ///
    /// Typically `[[ ! condition ]]`
    Invert(Box<Condition>),
}
