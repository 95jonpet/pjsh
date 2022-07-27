use crate::Program;

/// A word represents a single unit of input and are commonly used for
/// identifiers, program names, and program arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word {
    /// A literal, whitespace separated, word.
    Literal(String),

    /// A quoted word.
    Quoted(String),

    /// A variable word name for a value that is resolved at runtime.
    Variable(String),

    /// Substitute the word with the evaluated value of a subshell.
    Subshell(Program),

    /// Substitute the word with the path to a temporary file consisting of the
    /// output from a program.
    ProcessSubstitution(Program),

    /// A complex word containing interpolable sub-units.
    Interpolation(Vec<InterpolationUnit>),
}

/// Interpolation units are sub-units of interpolable words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolationUnit {
    /// A literal interpolation unit.
    Literal(String),

    /// A unicode character.
    Unicode(char),

    /// A variable name for a value that is resolved at runtime.
    Variable(String),

    /// Substitute the interpolation unit with the output from a subshell.
    Subshell(Program),
}
