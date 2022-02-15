#[derive(Debug, PartialEq)]
pub enum Word<'a> {
    Literal(&'a str),
    Quoted(&'a str),
    TripleQuoted(&'a str),
    Variable(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct Command<'a>(pub Vec<Word<'a>>);

#[derive(Debug, PartialEq)]
pub struct Condition<'a>(pub Vec<Word<'a>>);

#[derive(Debug, PartialEq)]
pub struct Redirect<'a> {
    pub source: FileDescriptor<'a>,
    pub method: RedirectMethod,
    pub target: FileDescriptor<'a>,
}

#[derive(Debug, PartialEq)]
pub enum RedirectMethod {
    Write,
    Append,
}

#[derive(Debug, PartialEq)]
pub enum FileDescriptor<'a> {
    Numbered(usize),
    Named(Word<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Pipeline<'a> {
    pub is_async: bool,
    pub commands: Vec<Command<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Expression<'a> {
    And(Box<Expression<'a>>, Box<Expression<'a>>),
    Or(Box<Expression<'a>>, Box<Expression<'a>>),
    Pipeline(Pipeline<'a>),
    Condition(Condition<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Assignment<'a> {
    key: Word<'a>,
    value: Word<'a>,
    scope: AssignmentScope,
}

#[derive(Debug, PartialEq)]
pub enum AssignmentScope {
    /// Export the assignment to the current execution environment as an environment
    /// variable.
    Export,
    /// Assign the variable in the global scope in the current file or session.
    Global,
    /// Assign the variable in the current scope.
    Scoped,
}
