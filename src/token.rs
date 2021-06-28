#[derive(Debug, PartialEq)]
pub enum Token {
    /* Fundamental symbols. */
    Word(String),
    AssignmentWord(String, String),
    Name(String),
    Newline,
    IoNumber,

    /* Delimiters. */
    SQuote,
    DQuote,

    /* Operators containing a single character. */
    Pipe,
    LParen,
    RParen,
    Less,
    Great,
    And,
    Semi,

    /* Operators containing more than one character. */
    AndIf,
    OrIf,
    DSemi,
    DLess,
    DGreat,
    LessAnd,
    GreatAnd,
    LessGreat,
    DLessDash,
    Clobber,
    If,
    Then,
    Else,
    Elif,
    Fi,
    Do,
    Done,
    Case,
    Esac,
    While,
    Until,
    For,

    /* Reserved words. */
    LBrace,
    RBrace,
    Bang,
    In,

    /* Pseudotokens. */
    EOF,
    Unknown,
}
