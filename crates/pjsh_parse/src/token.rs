use crate::Span;

/// A unit of input identified through lexical analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token contents.
    pub contents: TokenContents,

    /// Token position in the input.
    pub span: Span,
}

impl Token {
    /// Constructs a new token.
    pub fn new(contents: TokenContents, span: Span) -> Self {
        Self { contents, span }
    }
}

/// The contents of a token.
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

    /// "$("
    DollarOpenParen,
    /// "("
    OpenParen,
    /// ")"
    CloseParen,
    /// "${"
    DollarOpenBrace,
    /// "{"
    OpenBrace,
    /// "}"
    CloseBrace,
    /// "["
    OpenBracket,
    /// "]"
    CloseBracket,
    /// "[["
    DoubleOpenBracket,
    /// "]]"
    DoubleCloseBracket,

    /// "&&"
    AndIf,
    /// "||"
    OrIf,

    /// "&"
    Amp,
    /// ":="
    Assign,
    /// "::="
    AssignResult,
    /// "|"
    Pipe,
    /// "->|"
    PipeStart,
    /// ";"
    Semi,
    /// "..."
    Spread,

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

/// An interpolation unit within an interpolation token.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationUnit {
    /// A literal unit.
    Literal(String),

    /// A unicode character.
    Unicode(char),

    /// The name of a variable unit that is evaluated at runtime.
    Variable(String),

    /// A value pipeline inside an interpolation.
    ValuePipeline(Vec<Token>),

    /// A subshell that is evaluated at runtime.
    Subshell(Vec<Token>),
}
