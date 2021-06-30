use crate::cursor::{Cursor, EOF_CHAR};
use crate::token::Token;

/// Returns the next [`Token`] in unquoted mode.
pub(crate) fn next_unquoted_token(cursor: &mut Cursor) -> Token {
    cursor.read_until(|ch| !ch.is_whitespace()); // Skip whitespaces.

    match *cursor.peek() {
        EOF_CHAR => Token::EOF, // End of input.
        '#' => {
            cursor.read_until(|ch| ch == &'\n');
            if let '\n' = cursor.peek() {
                cursor.next();
                Token::Newline
            } else {
                Token::EOF
            }
        }
        '\n' => {
            cursor.next();
            Token::Newline
        }
        '\'' => {
            cursor.next();
            Token::SQuote
        }
        '|' => {
            cursor.next();
            match cursor.peek() {
                '|' => {
                    cursor.next();
                    Token::OrIf
                }
                _ => Token::Pipe,
            }
        }
        '&' => {
            cursor.next();
            match cursor.peek() {
                '&' => {
                    cursor.next();
                    Token::AndIf
                }
                _ => Token::And,
            }
        }
        '(' => {
            cursor.next();
            Token::LParen
        }
        ')' => {
            cursor.next();
            Token::RParen
        }
        '<' => {
            cursor.next();
            match cursor.peek() {
                '<' => {
                    cursor.next();
                    match cursor.peek() {
                        '-' => {
                            cursor.next();
                            Token::DLessDash
                        }
                        _ => Token::DLess,
                    }
                }
                '&' => {
                    cursor.next();
                    Token::LessAnd
                }
                '>' => {
                    cursor.next();
                    Token::LessGreat
                }
                _ => Token::Less,
            }
        }
        '>' => {
            cursor.next();
            match cursor.peek() {
                '>' => {
                    cursor.next();
                    Token::DGreat
                }
                '&' => {
                    cursor.next();
                    Token::GreatAnd
                }
                '|' => {
                    cursor.next();
                    Token::Clobber
                }
                _ => Token::Great,
            }
        }
        ';' => {
            cursor.next();
            match cursor.peek() {
                ';' => {
                    cursor.next();
                    Token::DSemi
                }
                _ => Token::Semi,
            }
        }
        ch if ch.is_ascii_alphanumeric() => {
            let word = cursor.read_until(|ch| ch.is_ascii_whitespace());
            delimit_keyword_token(&word).unwrap_or_else(|| Token::Word(word))
        }
        _ => unimplemented!("Cannot yet handle `{}`.", cursor.peek()),
    }
}

/// Returns a delimited [`Token`] for a word denoting a keyword.
fn delimit_keyword_token(word: &str) -> Option<Token> {
    match word {
        // "if" => Some(Token::If),
        // "then" => Some(Token::Then),
        // "else" => Some(Token::Else),
        // "elif" => Some(Token::Elif),
        // "fi" => Some(Token::Fi),
        // "do" => Some(Token::Do),
        // "done" => Some(Token::Done),
        // "case" => Some(Token::Case),
        // "esac" => Some(Token::Esac),
        // "while" => Some(Token::While),
        // "until" => Some(Token::Until),
        // "for" => Some(Token::For),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        input::InputLines,
        lexer::{Lex, Lexer, Mode},
    };

    use super::*;

    #[test]
    fn it_ignores_comments() {
        assert_eq!(
            lex("code # comment."),
            vec![Token::Word("code".to_string())]
        );
        assert_eq!(
            lex("code # Newline is kept\ntest"),
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
            lex("first second"),
            vec![
                Token::Word("first".to_string()),
                Token::Word("second".to_string())
            ]
        );
        assert_eq!(
            lex("first    second"),
            vec![
                Token::Word("first".to_string()),
                Token::Word("second".to_string())
            ]
        );
    }

    #[test]
    fn it_splits_words_on_tabs() {
        assert_eq!(
            lex("first\tsecond"),
            vec![
                Token::Word("first".to_string()),
                Token::Word("second".to_string())
            ]
        );
        assert_eq!(
            lex("first\t\tsecond"),
            vec![
                Token::Word("first".to_string()),
                Token::Word("second".to_string())
            ]
        );
    }

    #[test]
    fn it_identifies_pipe_tokens() {
        assert_eq!(lex("|"), vec![Token::Pipe]);
    }

    #[test]
    fn it_identifies_lparen_tokens() {
        assert_eq!(lex("("), vec![Token::LParen]);
    }

    #[test]
    fn it_identifies_rparen_tokens() {
        assert_eq!(lex(")"), vec![Token::RParen]);
    }

    #[test]
    fn it_identifies_less_tokens() {
        assert_eq!(lex("<"), vec![Token::Less]);
    }

    #[test]
    fn it_identifies_great_tokens() {
        assert_eq!(lex(">"), vec![Token::Great]);
    }

    #[test]
    fn it_identifies_and_tokens() {
        assert_eq!(lex("&"), vec![Token::And]);
    }

    #[test]
    fn it_identifies_semi_tokens() {
        assert_eq!(lex(";"), vec![Token::Semi]);
    }

    #[test]
    fn it_identifies_andif_tokens() {
        assert_eq!(lex("&&"), vec![Token::AndIf]);
    }

    #[test]
    fn it_identifies_orif_tokens() {
        assert_eq!(lex("||"), vec![Token::OrIf]);
    }

    #[test]
    fn it_identifies_dsemi_tokens() {
        assert_eq!(lex(";;"), vec![Token::DSemi]);
    }

    #[test]
    fn it_identifies_dless_tokens() {
        assert_eq!(lex("<<"), vec![Token::DLess]);
    }

    #[test]
    fn it_identifies_dgreat_tokens() {
        assert_eq!(lex(">>"), vec![Token::DGreat]);
    }

    #[test]
    fn it_identifies_lessand_tokens() {
        assert_eq!(lex("<&"), vec![Token::LessAnd]);
    }

    #[test]
    fn it_identifies_greatand_tokens() {
        assert_eq!(lex(">&"), vec![Token::GreatAnd]);
    }

    #[test]
    fn it_identifies_lessgreat_tokens() {
        assert_eq!(lex("<>"), vec![Token::LessGreat]);
    }

    #[test]
    fn it_identifies_dlessdash_tokens() {
        assert_eq!(lex("<<-"), vec![Token::DLessDash]);
    }

    #[test]
    fn it_identifies_clobber_tokens() {
        assert_eq!(lex(">|"), vec![Token::Clobber]);
    }

    fn lex(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut lexer = Lexer {
            cursor: Cursor::new(InputLines::Single(Some(String::from(input)))),
        };

        loop {
            let token = lexer.next_token(Mode::Unquoted);
            if token == Token::EOF {
                break;
            }
            tokens.push(token);
        }

        tokens
    }
}
