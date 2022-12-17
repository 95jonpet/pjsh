use crate::{Filter, Program};

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

    /// A complex value-based pipeline.
    ValuePipeline(Box<ValuePipeline>),
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

    /// A value-based pipeline.
    ValuePipeline(ValuePipeline),

    /// Substitute the interpolation unit with the output from a subshell.
    Subshell(Program),
}

/// A value-based pipeline resulting in a single value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValuePipeline {
    /// Base value reference (a variable name).
    pub base: String,

    /// Filters to run value and its resultant values through.
    pub filters: Vec<Filter>,
}
