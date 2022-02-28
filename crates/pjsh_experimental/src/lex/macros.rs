#[macro_export]
macro_rules! create_punctuation_kind {
    (Separator) => {
        crate::lex::lexer::PunctuationKind::Separator
    };
    (Open $depth:expr) => {
        crate::lex::lexer::PunctuationKind::Open($depth)
    };
    (Close $depth:expr) => {
        crate::lex::lexer::PunctuationKind::Close($depth)
    };
}

#[macro_export]
macro_rules! tok {
    (Eof) => {
        crate::lex::lexer::Token::Eof
    };
    (Char $raw:tt) => {
        crate::lex::lexer::Token::Char($raw)
    };
    (Punc $raw:tt ($($inner:tt)+)) => {
        crate::lex::lexer::TokenType::Punctuation{ raw: $raw, kind: create_punctuation_kind!($($inner) +) }
    };
}
