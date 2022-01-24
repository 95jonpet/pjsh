use crate::{AndOr, Program};

/// Represents a chain of conditional, "if", statements.
///
/// A condition chain consists of a set of conditions with matching branches.
/// There is exactly one branch per condition plus an optional "else" branch
/// that is executed if no other condition is met. Thus, for `n` conditions,
/// there can be either `n` or `n+1` branches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalChain {
    pub conditions: Vec<AndOr>,
    pub branches: Vec<Program>,
}
