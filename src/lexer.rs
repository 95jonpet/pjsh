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

    pub fn next_token(&mut self, mode: &Mode) -> Token {
        match mode {
            Mode::InSingleQuotes => self.next_token_in_single_quotes(),
            Mode::Unquoted => {
                self.read_until(|ch| !ch.is_whitespace()); // Skip whitespaces.
                let mut keyword_possible = true;

                match *self.cursor.peek() {
                    EOF_CHAR => Token::EOF, // End of input.
                    '#' => {
                        self.read_until(|ch| ch == &'\n');
                        if let '\n' = self.cursor.peek() {
                            self.cursor.next();
                            Token::Newline
                        } else {
                            Token::EOF
                        }
                    }
                    '\n' => {
                        self.cursor.next();
                        Token::Newline
                    }
                    '\'' => {
                        self.cursor.next();
                        Token::SQuote
                    }
                    '|' => {
                        self.cursor.next();
                        match self.cursor.peek() {
                            '|' => {
                                self.cursor.next();
                                Token::OrIf
                            }
                            _ => Token::Pipe
                        }
                    }
                    '&' => {
                        self.cursor.next();
                        match self.cursor.peek() {
                            '&' => {
                                self.cursor.next();
                                Token::AndIf
                            }
                            _ => Token::And
                        }
                    }
                    '(' => {
                        self.cursor.next();
                        Token::LParen
                    }
                    ')' => {
                        self.cursor.next();
                        Token::RParen
                    }
                    '<' => {
                        self.cursor.next();
                        match self.cursor.peek() {
                            '<' => {
                                self.cursor.next();
                                match self.cursor.peek() {
                                    '-' => {
                                        self.cursor.next();
                                        Token::DLessDash
                                    }
                                    _ => Token::DLess
                                }
                            }
                            '&' => {
                                self.cursor.next();
                                Token::LessAnd
                            }
                            '>' => {
                                self.cursor.next();
                                Token::LessGreat
                            }
                            _ => Token::Less
                        }
                    }
                    '>' => {
                        self.cursor.next();
                        match self.cursor.peek() {
                            '>' => {
                                self.cursor.next();
                                Token::DGreat
                            }
                            '&' => {
                                self.cursor.next();
                                Token::GreatAnd
                            }
                            '|' => {
                                self.cursor.next();
                                Token::Clobber
                            }
                            _ => Token::Great
                        }
                    }
                    ';' => {
                        self.cursor.next();
                        match self.cursor.peek() {
                            ';' => {
                                self.cursor.next();
                                Token::DSemi
                            }
                            _ => Token::Semi
                        }
                    }
                    ch if ch.is_ascii_alphanumeric() => {
                        let word = self.read_until(|ch| ch.is_ascii_whitespace());
                        self.delimit_word_token(word)
                    }
                    _ => unimplemented!("Cannot yet handle `{}`.", self.cursor.peek()),
                }
            }
        }
    }

    fn next_token_in_single_quotes(&mut self) -> Token {
        match *self.cursor.peek() {
            EOF_CHAR => Token::EOF, // End of input.
            '\'' => {
                self.cursor.next();
                Token::SQuote
            }
            _ => Token::Word(self.read_until(|ch| ch == &'\'')),
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

    fn delimit_word_token(&self, word: String) -> Token {
        self.operator_token(&word)
            .unwrap_or_else(|| Token::Word(word))
    }

    fn operator_token(&self, lexeme: &str) -> Option<Token> {
        match lexeme {
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

    #[test]
    fn it_tokenizes_operators() {
        assert_eq!(lex(Mode::Unquoted, "|"), vec![Token::Pipe]);
        assert_eq!(lex(Mode::Unquoted, "("), vec![Token::LParen]);
        assert_eq!(lex(Mode::Unquoted, ")"), vec![Token::RParen]);
        assert_eq!(lex(Mode::Unquoted, "<"), vec![Token::Less]);
        assert_eq!(lex(Mode::Unquoted, ">"), vec![Token::Great]);
        assert_eq!(lex(Mode::Unquoted, "&"), vec![Token::And]);
        assert_eq!(lex(Mode::Unquoted, ";"), vec![Token::Semi]);
        assert_eq!(lex(Mode::Unquoted, "&&"), vec![Token::AndIf]);
        assert_eq!(lex(Mode::Unquoted, "||"), vec![Token::OrIf]);
        assert_eq!(lex(Mode::Unquoted, ";;"), vec![Token::DSemi]);
        assert_eq!(lex(Mode::Unquoted, "<<"), vec![Token::DLess]);
        assert_eq!(lex(Mode::Unquoted, ">>"), vec![Token::DGreat]);
        assert_eq!(lex(Mode::Unquoted, "<&"), vec![Token::LessAnd]);
        assert_eq!(lex(Mode::Unquoted, ">&"), vec![Token::GreatAnd]);
        assert_eq!(lex(Mode::Unquoted, "<>"), vec![Token::LessGreat]);
        assert_eq!(lex(Mode::Unquoted, "<<-"), vec![Token::DLessDash]);
        assert_eq!(lex(Mode::Unquoted, ">|"), vec![Token::Clobber]);
        assert_eq!(lex(Mode::Unquoted, "if"), vec![Token::If]);
        assert_eq!(lex(Mode::Unquoted, "then"), vec![Token::Then]);
        assert_eq!(lex(Mode::Unquoted, "else"), vec![Token::Else]);
        assert_eq!(lex(Mode::Unquoted, "elif"), vec![Token::Elif]);
        assert_eq!(lex(Mode::Unquoted, "fi"), vec![Token::Fi]);
        assert_eq!(lex(Mode::Unquoted, "do"), vec![Token::Do]);
        assert_eq!(lex(Mode::Unquoted, "done"), vec![Token::Done]);
        assert_eq!(lex(Mode::Unquoted, "case"), vec![Token::Case]);
        assert_eq!(lex(Mode::Unquoted, "esac"), vec![Token::Esac]);
        assert_eq!(lex(Mode::Unquoted, "while"), vec![Token::While]);
        assert_eq!(lex(Mode::Unquoted, "until"), vec![Token::Until]);
        assert_eq!(lex(Mode::Unquoted, "for"), vec![Token::For]);
    }

    fn lex(mode: Mode, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut lexer = Lexer {
            cursor: Cursor::new(InputLines::Single(Some(String::from(input)))),
        };

        loop {
            let token = lexer.next_token(&mode);
            if token == Token::EOF {
                break;
            }
            tokens.push(token);
        }

        tokens
    }
}
