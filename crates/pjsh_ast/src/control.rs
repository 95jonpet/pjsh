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

/// Represents a piece of code that is repeatedly executed for as long as a
/// condition is met.
///
/// The condition is always checked prior to executing the body. Thus, if the
/// condition is never met, the body is never entered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalLoop {
    pub condition: AndOr,
    pub body: Program,
}
