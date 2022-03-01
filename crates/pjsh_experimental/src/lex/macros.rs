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
        crate::lex::lexer::TokenType::Eof
    };
    (Char $raw:tt) => {
        crate::lex::lexer::TokenType::Char($raw)
    };
    (Op $raw:tt) => {
        crate::lex::lexer::TokenType::Operator($raw)
    };
    (Id $raw:tt) => {
        crate::lex::lexer::TokenType::Identifier($raw)
    };
    (Str $raw:tt) => {
        crate::lex::lexer::TokenType::String($raw.into())
    };
    (Punct $raw:tt ($($inner:tt)+)) => {
        crate::lex::lexer::TokenType::Punctuation{ raw: $raw, kind: crate::create_punctuation_kind!($($inner) +) }
    };
}

#[cfg(test)]
mod tests {
    use crate::lex::lexer::{PunctuationKind, TokenType};

    #[test]
    fn it_creates_eof_tokens() {
        assert_eq!(TokenType::Eof, tok!(Eof));
    }

    #[test]
    fn it_creates_identifier_tokens() {
        assert_eq!(TokenType::Identifier("id"), tok!(Id "id"));
    }

    #[test]
    fn it_creates_string_tokens() {
        assert_eq!(
            TokenType::String("This is a string.".into()),
            tok!(Str "This is a string.")
        );
    }

    #[test]
    fn it_creates_punctuation() {
        assert_eq!(
            TokenType::Punctuation {
                raw: '(',
                kind: PunctuationKind::Open(0),
            },
            tok!(Punct '(' (Open 0))
        );
        assert_eq!(
            TokenType::Punctuation {
                raw: ')',
                kind: PunctuationKind::Close(1),
            },
            tok!(Punct ')' (Close 1))
        );
        assert_eq!(
            TokenType::Punctuation {
                raw: ';',
                kind: PunctuationKind::Separator
            },
            tok!(Punct ';' (Separator))
        );
    }
}
