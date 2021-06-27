use crate::{
    cursor::{Cursor, EOF_CHAR},
    token::Token,
};

pub(crate) enum Mode {
    Unquoted,
    InSingleQuotes,
    // InDoubleQuotes,
    // Arithmetic,
}

pub(crate) struct Lexer {
    cursor: Cursor,
}

impl Lexer {
    pub fn new(cursor: Cursor) -> Self {
        Self { cursor }
    }

    pub fn next_token(&mut self, mode: &Mode) -> Option<Token> {
        match mode {
            Mode::InSingleQuotes => self.next_token_in_single_quotes(),
            Mode::Unquoted => {
                self.read_until(|ch| !ch.is_whitespace()); // Skip whitespaces.

                match *self.cursor.peek() {
                    EOF_CHAR => None, // End of input.
                    '#' => {
                        self.read_until(|ch| ch == &'\n');
                        if let '\n' = self.cursor.peek() {
                            self.cursor.next();
                            Some(Token::Newline)
                        } else {
                            None
                        }
                    }
                    '\n' => {
                        self.cursor.next();
                        Some(Token::Newline)
                    }
                    '\'' => {
                        self.cursor.next();
                        Some(Token::SQuote)
                    }
                    ch if ch.is_ascii_alphanumeric() => {
                        Some(Token::Word(self.read_until(|ch| ch.is_ascii_whitespace())))
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }

    fn next_token_in_single_quotes(&mut self) -> Option<Token> {
        match *self.cursor.peek() {
            EOF_CHAR => None, // End of input.
            '\'' => {
                self.cursor.next();
                Some(Token::SQuote)
            }
            _ => Some(Token::Word(self.read_until(|ch| ch == &'\''))),
        }
    }

    fn read_until<P>(&mut self, predicate: P) -> String
    where
        P: Fn(&char) -> bool,
    {
        let mut result = String::new();
        loop {
            match self.cursor.peek() {
                ch if !predicate(ch) && ch != &EOF_CHAR => {
                    let c = self.cursor.next();
                    result.push(c);
                }
                _ => break,
            }
        }
        result
    }

    fn delimit_token(&self, lexeme: &str) -> Option<Token> {
        if lexeme.is_empty() {
            return None;
        }

        self.operator_token(lexeme)
    }

    fn operator_token(&self, lexeme: &str) -> Option<Token> {
        match lexeme {
            "|" => Some(Token::Pipe),
            "(" => Some(Token::LParen),
            ")" => Some(Token::RParen),
            "<" => Some(Token::Less),
            ">" => Some(Token::Great),
            "&" => Some(Token::And),
            ";" => Some(Token::Semi),

            "&&" => Some(Token::AndIf),
            "||" => Some(Token::OrIf),
            ";;" => Some(Token::DSemi),
            "<<" => Some(Token::DLess),
            ">>" => Some(Token::DGreat),
            "<&" => Some(Token::LessAnd),
            ">&" => Some(Token::GreatAnd),
            "<>" => Some(Token::LessGreat),
            "<<-" => Some(Token::DLessDash),
            ">|" => Some(Token::Clobber),
            "if" => Some(Token::If),
            "then" => Some(Token::Then),
            "else" => Some(Token::Else),
            "elif" => Some(Token::Elif),
            "fi" => Some(Token::Fi),
            "do" => Some(Token::Do),
            "done" => Some(Token::Done),
            "case" => Some(Token::Case),
            "esac" => Some(Token::Esac),
            "while" => Some(Token::While),
            "until" => Some(Token::Until),
            "for" => Some(Token::For),

            // "{" => Some(Token::LBrace),
            // "}" => Some(Token::RBrace),
            // "!" => Some(Token::Bang),
            // "in" => Some(Token::In),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::input::InputLines;

    use super::*;

    #[test]
    fn it_ignores_comments() {
        assert_eq!(
            lex(Mode::Unquoted, "code # comment."),
            vec![Token::Word("code".to_string())]
        );
        assert_eq!(
            lex(Mode::Unquoted, "code # Newline is kept\ntest"),
            vec![
                Token::Word("code".to_string()),
                Token::Newline,
                Token::Word("test".to_string()),
            ]
        );
    }

    #[test]
    fn it_splits_words_on_spaces() {
        assert_eq!(
            lex(Mode::Unquoted, "first second"),
            vec![
                Token::Word("first".to_string()),
                Token::Word("second".to_string())
            ]
        );
        assert_eq!(
            lex(Mode::Unquoted, "first    second"),
            vec![
                Token::Word("first".to_string()),
                Token::Word("second".to_string())
            ]
        );
    }

    #[test]
    fn it_splits_words_on_tabs() {
        assert_eq!(
            lex(Mode::Unquoted, "first\tsecond"),
            vec![
                Token::Word("first".to_string()),
                Token::Word("second".to_string())
            ]
        );
        assert_eq!(
            lex(Mode::Unquoted, "first\t\tsecond"),
            vec![
                Token::Word("first".to_string()),
                Token::Word("second".to_string())
            ]
        );
    }

    fn lex(mode: Mode, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut lexer = Lexer {
            cursor: Cursor::new(InputLines::Single(Some(String::from(input)))),
        };

        while let Some(token) = lexer.next_token(&mode) {
            tokens.push(token);
        }

        tokens
    }
}
