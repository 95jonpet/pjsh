#[derive(Debug, Clone, PartialEq)]
pub enum TokenContents<'a> {
    /// "# ..."
    Comment,

    Interpolation(Vec<InterpolationUnit<'a>>),
    Literal(&'a str),
    Variable(&'a str),
    Quoted(&'a str),

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
pub enum InterpolationUnit<'a> {
    Literal(&'a str),
    Unicode(char),
    Variable(&'a str),
}
