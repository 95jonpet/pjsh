use crate::{AndOr, Block, Iterable, Word};

/// Represents a chain of conditional, "if", statements.
///
/// A condition chain consists of a set of conditions with matching branches.
/// There is exactly one branch per condition plus an optional "else" branch
/// that is executed if no other condition is met. Thus, for `n` conditions,
/// there can be either `n` or `n+1` branches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalChain {
    /// Branch conditions.
    pub conditions: Vec<AndOr>,

    /// Conditional branches.
    ///
    /// The `n`-th branch is only executed if the `n`-th branch condition is met.
    pub branches: Vec<Block>,
}

/// Represents a piece of code that is repeatedly executed for as long as a
/// condition is met.
///
/// The condition is always checked prior to executing the body. Thus, if the
/// condition is never met, the body is never entered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalLoop {
    /// Loop condition.
    ///
    /// The body is executed for as long as this condition holds. The condition is
    /// always evaluated before the body is executed.
    pub condition: AndOr,

    /// Loop body.
    pub body: Block,
}

/// Represents a piece of code that is executed once for each item in an iterator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForIterableLoop {
    /// Loop variable name.
    pub variable: String,

    /// Iterable.
    pub iterable: Iterable,

    /// Loop body.
    pub body: Block,
}

/// Represents a piece of code that is executed once for each item in an
/// abstract iterator.
///
/// This type of loop is typically coerced into a [`ForIterableLoop`] when a
/// context is supplied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForOfIterableLoop {
    /// Loop variable name.
    pub variable: String,

    /// Abstract iteration rule.
    pub iteration_rule: IterationRule,

    /// Iterable.
    pub iterable: Word,

    /// Loop body.
    pub body: Block,
}

/// An abstract iteration rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IterationRule {
    /// Iterate over characters.
    Chars,

    /// Iterate over lines.
    Lines,

    /// Iterate over whitespace-separated words.
    Words,
}

/// Represents a conditional, "switch", statements over multiple values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Switch {
    /// The input word to match.
    pub input: Word,

    /// Branches to execute conditionally based on input.
    ///
    /// A branch is executed if its word matches the input.
    pub branches: Vec<(Word, Block)>,
}
