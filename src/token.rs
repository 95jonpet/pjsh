#[derive(Debug, PartialEq)]
pub enum Token {
    Word(String),
    Keyword(Keyword),
    Separator(Separator),
    Operator(Operator),
    Comment(String),
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
    Assign,
    Bang,
    Equal,
    Or,
    Pipe,
}
