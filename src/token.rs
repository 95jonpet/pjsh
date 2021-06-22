#[derive(Debug, PartialEq)]
pub enum Token {
    Word(Vec<Unit>),
    Keyword(Keyword),
    Separator(Separator),
    Operator(Operator),
    Assign(String, String),
    Comment(String),
}

#[derive(Debug, PartialEq)]
pub enum Unit {
    Literal(String),
    Variable(String),
}

#[derive(Debug, PartialEq)]
pub enum Separator {
    Semicolon,
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Case,
    Do,
    Done,
    Elif,
    Else,
    Esac,
    Fi,
    For,
    If,
    In,
    Then,
    Until,
    While,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Ampersand,
    And,
    Bang,
    Equal,
    Or,
    Pipe,
}
