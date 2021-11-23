use crate::Statement;

#[derive(Debug, PartialEq, Eq)]
pub struct Program<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl<'a> Program<'a> {
    /// Constructs a new empty program.
    pub fn new() -> Self {
        let statements = Vec::new();
        Self { statements }
    }

    /// Appends a statement to the program.
    pub fn statement<'b>(&'b mut self, statement: Statement<'a>) -> &'b mut Self {
        self.statements.push(statement);
        self
    }
}

impl Default for Program<'_> {
    fn default() -> Self {
        Self::new()
    }
}
