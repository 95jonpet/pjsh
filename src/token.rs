use std::fmt::Display;

/// Represents a sequence of characters which holds special meaning for a parser.
/// Typically generated by a lexer.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /* Fundamental symbols. */
    /// Generic word.
    /// This symbol may be resolved to other subtokens such as assignment words and names.
    Word(Vec<Unit>),
    /// \n
    Newline,
    /// Integer preceding redirection operators.
    IoNumber(u8),

    /* Operators containing a single character. */
    /// |
    Pipe,
    /// (
    LParen,
    /// )
    RParen,
    /// <
    Less,
    /// >
    Great,
    /// &
    And,
    /// ;
    Semi,

    /* Operators containing more than one character. */
    /// &&
    AndIf,
    /// ||
    OrIf,
    /// ;;
    DSemi,
    /// <<
    DLess,
    /// >>
    DGreat,
    /// <&
    LessAnd,
    /// >&
    GreatAnd,
    /// <>
    LessGreat,
    /// <<-
    DLessDash,
    /// >|
    Clobber,

    /* Pseudotokens. */
    /// End of file. Typically \0.
    Eof,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Word(units) => write!(f, "<word {:?}>", units),
            Token::Newline => write!(f, "<newline>"),
            Token::IoNumber(number) => write!(f, "<io {}>", number),
            Token::Pipe => write!(f, "|"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Less => write!(f, "<"),
            Token::Great => write!(f, ">"),
            Token::And => write!(f, "&"),
            Token::Semi => write!(f, ";"),
            Token::AndIf => write!(f, "&&"),
            Token::OrIf => write!(f, "||"),
            Token::DSemi => write!(f, ";;"),
            Token::DLess => write!(f, "<<"),
            Token::DGreat => write!(f, ">>"),
            Token::LessAnd => write!(f, "<&"),
            Token::GreatAnd => write!(f, ">&"),
            Token::LessGreat => write!(f, "<>"),
            Token::DLessDash => write!(f, "<<-"),
            Token::Clobber => write!(f, ">|"),
            Token::Eof => write!(f, "<EOF>"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Unit {
    Literal(String),
    Var(String),
    Expression(Expression),
}

// TODO: Use Option<Token::Word> for optional word arguments to allow word expansion.
#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    /// parameter: ${parameter}
    Parameter(String),
    /// parameter, word, unset_or_null: ${parameter:-[word]}
    UseDefaultValues(String, String, bool),
    /// parameter, word, unset_or_null: ${parameter:=[word]}
    AssignDefaultValues(String, String, bool),
    /// parameter, word, unset_or_null: ${parameter:?[word]}
    IndicateError(String, String, bool),
    /// parameter, word, unset_or_null: ${parameter:+[word]}
    UseAlternativeValue(String, String, bool),
}
