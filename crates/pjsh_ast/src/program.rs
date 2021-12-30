use crate::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
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
