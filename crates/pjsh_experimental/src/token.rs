#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub span: Span,
    pub it: TokenContents<'a>,
}

impl<'a> Token<'a> {
    pub fn new(span: Span, it: TokenContents<'a>) -> Self {
        Self { span, it }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenContents<'a> {
    /// "# ..."
    Comment,

    Literal(&'a str),
    Quoted(&'a str),
    Variable(&'a str),

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
