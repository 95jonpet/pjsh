#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Keyword(Keyword),
    Separator(Separator),
    Operator(Operator),
    Literal(Literal),
    Comment(String),
}

#[derive(Debug, PartialEq)]
pub enum Separator {
    Semicolon,
    SingleQuote,
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
    String(String),
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
    Equal,
    Or,
    Pipe,
}
