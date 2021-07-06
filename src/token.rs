#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /* Fundamental symbols. */
    /// Generic word.
    /// This symbol may be resolved to other subtokens such as assignment words and names.
    Word(String),
    /// \n
    Newline,
    /// Integer preceding redirection operators.
    IoNumber(u8),

    /* Delimiters. */
    /// '
    SQuote,
    /// "
    DQuote,

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
    /// if
    If,
    /// then
    Then,
    /// else
    Else,
    /// elif
    Elif,
    /// fi
    Fi,
    /// do
    Do,
    /// done
    Done,
    /// case
    Case,
    /// esac
    Esac,
    /// while
    While,
    /// until
    Until,
    /// for
    For,

    /* Reserved words. */
    // LBrace,
    // RBrace,
    // Bang,
    // In,

    /* Pseudotokens. */
    EOF,
}
