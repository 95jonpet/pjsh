use crate::cursor::{Cursor, EOF_CHAR};
use crate::token::Token;

/// Returns the next [`Token`] in unquoted mode.
pub(crate) fn next_unquoted_token(cursor: &mut Cursor) -> Token {
    // cursor.read_until(|ch| !ch.is_whitespace()); // Skip whitespaces.
    cursor.read_until(|ch| ch != &' ' && ch != &'\t' && ch != &'\r'); // Skip whitespaces.

    match *cursor.peek() {
        EOF_CHAR => Token::EOF, // End of input.
        '#' => {
            cursor.read_until(|ch| ch == &'\n');
            if let '\n' = cursor.peek() {
                cursor.next();
                if cursor.is_interactive() {
                    Token::EOF // A complete command has been sent.
                } else {
                    Token::Newline
                }
            } else {
                Token::EOF
            }
        }
        '\n' => {
            cursor.next();
            if cursor.is_interactive() {
                Token::EOF // A complete command has been sent.
            } else {
                Token::Newline
            }
        }
        '\'' => {
            cursor.next();
            Token::SQuote
        }
        '"' => {
            cursor.next();
            Token::DQuote
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
        ch if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '/' => {
            let word = cursor.read_until(|ch| {
                !ch.is_ascii_alphanumeric() && ch != &'_' && ch != &'-' && ch != &'/'
            });
            match cursor.peek() {
                &'<' | &'>' if word.parse::<u8>().is_ok() => {
                    Token::IoNumber(word.parse::<u8>().unwrap())
                }
                _ => Token::Word(word),
            }
        }
        _ => unimplemented!("Cannot yet handle `{}`.", cursor.peek()),
    }
}

// #[cfg(test)]
// mod tests {
//     use std::{cell::RefCell, rc::Rc};

//     use crate::{
//         input::InputLines,
//         lexer::{Lex, Lexer, Mode},
//         options::Options,
//     };

//     use super::*;

//     #[test]
//     fn it_ignores_comments() {
//         assert_eq!(
//             lex("code # comment."),
//             vec![Token::Word("code".to_string())]
//         );
//         assert_eq!(
//             lex("code # Newline is kept\ntest"),
//             vec![
//                 Token::Word("code".to_string()),
//                 Token::Newline,
//                 Token::Word("test".to_string()),
//             ]
//         );
//     }

//     #[test]
//     fn it_splits_words_on_spaces() {
//         assert_eq!(
//             lex("first second"),
//             vec![
//                 Token::Word("first".to_string()),
//                 Token::Word("second".to_string())
//             ]
//         );
//         assert_eq!(
//             lex("first    second"),
//             vec![
//                 Token::Word("first".to_string()),
//                 Token::Word("second".to_string())
//             ]
//         );
//     }

//     #[test]
//     fn it_splits_words_on_tabs() {
//         assert_eq!(
//             lex("first\tsecond"),
//             vec![
//                 Token::Word("first".to_string()),
//                 Token::Word("second".to_string())
//             ]
//         );
//         assert_eq!(
//             lex("first\t\tsecond"),
//             vec![
//                 Token::Word("first".to_string()),
//                 Token::Word("second".to_string())
//             ]
//         );
//     }

//     #[test]
//     fn it_identifies_io_number_tokens() {
//         assert_eq!(lex("2>"), vec![Token::IoNumber(2), Token::Great]);
//         assert_eq!(
//             lex("1< file"),
//             vec![
//                 Token::IoNumber(1),
//                 Token::Less,
//                 Token::Word(String::from("file"))
//             ]
//         );
//     }

//     #[test]
//     fn it_identifies_pipe_tokens() {
//         assert_eq!(lex("|"), vec![Token::Pipe]);
//     }

//     #[test]
//     fn it_identifies_lparen_tokens() {
//         assert_eq!(lex("("), vec![Token::LParen]);
//     }

//     #[test]
//     fn it_identifies_rparen_tokens() {
//         assert_eq!(lex(")"), vec![Token::RParen]);
//     }

//     #[test]
//     fn it_identifies_less_tokens() {
//         assert_eq!(lex("<"), vec![Token::Less]);
//     }

//     #[test]
//     fn it_identifies_great_tokens() {
//         assert_eq!(lex(">"), vec![Token::Great]);
//     }

//     #[test]
//     fn it_identifies_and_tokens() {
//         assert_eq!(lex("&"), vec![Token::And]);
//     }

//     #[test]
//     fn it_identifies_semi_tokens() {
//         assert_eq!(lex(";"), vec![Token::Semi]);
//     }

//     #[test]
//     fn it_identifies_andif_tokens() {
//         assert_eq!(lex("&&"), vec![Token::AndIf]);
//     }

//     #[test]
//     fn it_identifies_orif_tokens() {
//         assert_eq!(lex("||"), vec![Token::OrIf]);
//     }

//     #[test]
//     fn it_identifies_dsemi_tokens() {
//         assert_eq!(lex(";;"), vec![Token::DSemi]);
//     }

//     #[test]
//     fn it_identifies_dless_tokens() {
//         assert_eq!(lex("<<"), vec![Token::DLess]);
//     }

//     #[test]
//     fn it_identifies_dgreat_tokens() {
//         assert_eq!(lex(">>"), vec![Token::DGreat]);
//     }

//     #[test]
//     fn it_identifies_lessand_tokens() {
//         assert_eq!(lex("<&"), vec![Token::LessAnd]);
//     }

//     #[test]
//     fn it_identifies_greatand_tokens() {
//         assert_eq!(lex(">&"), vec![Token::GreatAnd]);
//     }

//     #[test]
//     fn it_identifies_lessgreat_tokens() {
//         assert_eq!(lex("<>"), vec![Token::LessGreat]);
//     }

//     #[test]
//     fn it_identifies_dlessdash_tokens() {
//         assert_eq!(lex("<<-"), vec![Token::DLessDash]);
//     }

//     #[test]
//     fn it_identifies_clobber_tokens() {
//         assert_eq!(lex(">|"), vec![Token::Clobber]);
//     }

//     fn lex(input: &str) -> Vec<Token> {
//         let mut tokens = Vec::new();
//         let options = Rc::new(RefCell::new(Options::default()));
//         let mut lexer = Lexer {
//             cursor: Cursor::new(
//                 InputLines::Single(Some(String::from(input))),
//                 false,
//                 options.clone(),
//             ),
//             options: options.clone(),
//         };

//         loop {
//             let token = lexer.next_token(Mode::Unquoted);
//             if token == Token::EOF {
//                 break;
//             }
//             tokens.push(token);
//         }

//         tokens
//     }
// }
