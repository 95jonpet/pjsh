use crate::lex::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenContents {
    /// "# ..."
    Comment,

    Interpolation(Vec<InterpolationUnit>),
    Literal(String),
    Variable(String),
    Quoted(String),

    /// "\"" or "'"
    Quote,
    /// "\"\"\"" or "'''"
    TripleQuote,

    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "[["
    DoubleOpenBracket,
    /// "]}"
    DoubleCloseBracket,

    /// "&&"
    AndIf,
    /// "||"
    OrIf,

    /// "&"
    Amp,
    /// ":="
    Assign,
    /// "|"
    Pipe,
    /// "->|"
    PipeStart,
    /// ";"
    Semi,

    /// "<"
    FdReadTo(usize),
    /// ">"
    FdWriteFrom(usize),
    /// ">>"
    FdAppendFrom(usize),

    /// ","
    Comma,
    /// "="
    Equal,

    /// "<("
    ProcessSubstitutionStart,

    /// End of line.
    /// "\n", "\r\n"
    Eol,
    /// " ", "\t"
    Whitespace,

    /// End of file.
    /// "\0"
    Eof,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationUnit {
    Literal(String),
    Unicode(char),
    Variable(String),
    Subshell(Vec<Token>),
}
