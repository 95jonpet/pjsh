use crate::{
    ConditionalChain, ConditionalLoop, ForIterableLoop, ForOfIterableLoop, List, Pipeline, Word,
};

/// A statement is an evaluable and/or executable piece of code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    /// Pipelines to executed conditionally.
    AndOr(AndOr),

    /// A variable assignment.
    Assignment(Assignment),

    /// A function definition.
    Function(Function),

    /// A conditional expression.
    If(ConditionalChain),

    /// An iterable loop.
    ForIn(ForIterableLoop),

    /// An iterable loop over an abstract iteration rule.
    ForOfIn(ForOfIterableLoop),

    /// A conditional loop.
    While(ConditionalLoop),

    /// A nested program body.
    Subshell(Program),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A list value with 0 or more elements.
    List(List),

    /// A word value.
    Word(Word),
}

/// Assigns a value to a named key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The name to assign the value to.
    pub key: Word,

    /// The value to assign.
    pub value: Value,
}

impl Assignment {
    /// Constructs a new assignment.
    pub fn new(key: Word, value: Value) -> Self {
        Self { key, value }
    }
}

/// A function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    /// The function name.
    pub name: String,

    /// Regular argument names.
    pub args: Vec<String>,

    /// List type argument name (may only be the final argument).
    pub list_arg: Option<String>,

    /// Function body.
    pub body: Block,
}

impl Function {
    /// Constructs a new function definition.
    pub fn new(name: String, args: Vec<String>, list_arg: Option<String>, body: Block) -> Self {
        Self {
            name,
            args,
            list_arg,
            body,
        }
    }
}

/// A construct for conditionally executing pipelines.
///
/// Pipelines are only executed up until the first failing condition. The first
/// pipeline is always executed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AndOr {
    /// Conditional operators.
    ///
    /// The `n`-th operator is used to determine whether or not the `(n+1)`-th
    /// pipeline should be executed.
    pub operators: Vec<AndOrOp>,

    /// Pipelines to conditionally execute.
    ///
    /// The first pipeline is always executed.
    pub pipelines: Vec<Pipeline>,
}

/// Conditional operator for executing a pipeline based upon the result of a
/// previous pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndOrOp {
    /// Execute the pipeline only if the previous pipeline within an [`AndOr`]
    /// construct exited with a success status.
    And,

    /// Execute the pipeline only if the previous pipeline within an [`AndOr`]
    /// construct exited with a non-success status.
    Or,
}

/// A code block is a sequence of statements that are executed within the same
/// scope.
///
/// Nested code blocks are supported and result in nested execution contexts.
///
/// Code blocks are typically surrounded by curly braces `{...}`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Block {
    /// Statements to execute within the program.
    pub statements: Vec<Statement>,
}

impl Block {
    /// Appends a statement to the end of a block.
    pub fn statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}

/// A program consists of multiple executable statements.
///
/// Programs are executed within separate context scopes. Nested programs are
/// supported using "subshells".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// Statements to execute within the program.
    pub statements: Vec<Statement>,
}

impl Program {
    /// Constructs a new empty program.
    pub fn new() -> Self {
        let statements = Vec::new();
        Self { statements }
    }

    /// Appends a statement to the program.
    pub fn statement(&mut self, statement: Statement) -> &mut Self {
        self.statements.push(statement);
        self
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_statements_can_be_appended() {
        let statement = Statement::Assignment(Assignment {
            key: Word::Literal("key".into()),
            value: Value::Word(Word::Literal("value".into())),
        });
        let mut block = Block::default();
        block.statement(statement.clone());
        assert_eq!(block.statements, vec![statement]);
    }

    #[test]
    fn program_statements_can_be_appended() {
        let statement = Statement::Assignment(Assignment {
            key: Word::Literal("key".into()),
            value: Value::Word(Word::Literal("value".into())),
        });
        let mut program = Program::default();
        program.statement(statement.clone());
        assert_eq!(program.statements, vec![statement]);
    }
}
