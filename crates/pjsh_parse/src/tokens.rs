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
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,

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
}
