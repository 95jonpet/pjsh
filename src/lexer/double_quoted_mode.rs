use crate::cursor::{Cursor, EOF_CHAR};
use crate::token::Token;

/// Returns the next [`Token`] in double-quoted mode.
pub(crate) fn next_double_quoted_token(cursor: &mut Cursor) -> Token {
    match *cursor.peek() {
        EOF_CHAR => Token::EOF, // End of input.
        '"' => {
            cursor.next();
            Token::DQuote
        }
        _ => Token::Word(cursor.read_until(|ch| ch == &'"')),
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        input::InputLines,
        lexer::{Lex, Lexer, Mode},
        options::Options,
    };

    use super::*;

    #[test]
    fn it_ignores_comments() {
        assert_eq!(
            lex("# Not a comment."),
            vec![Token::Word("# Not a comment.".to_string())]
        );
    }

    #[test]
    fn it_does_not_split_words_on_spaces() {
        assert_eq!(lex("one word"), vec![Token::Word("one word".to_string())]);
    }

    #[test]
    fn it_does_not_split_words_on_tabs() {
        assert_eq!(lex("one\tword"), vec![Token::Word("one\tword".to_string())]);
    }

    #[test]
    fn it_does_not_split_words_on_newline() {
        assert_eq!(lex("one\nline"), vec![Token::Word("one\nline".to_string())]);
    }

    #[test]
    fn it_treats_unquoted_keywords_as_literals() {
        let inputs = [
            // These are considered keywords in unquoted mode.
            "\'", "|", "(", ")", "<", ">", "&", ";", "&&", "||", ";;", "<<", ">>", "<&", ">&", "<>",
            "<<-", ">|",
        ];

        for input in inputs {
            // Should be considered words in double-quoted mode.
            assert_eq!(lex(input), vec![Token::Word(String::from(input))]);
        }
    }

    #[test]
    fn it_identifies_dquote_tokens() {
        assert_eq!(lex("\""), vec![Token::DQuote]);
        assert_eq!(
            lex("w\""),
            vec![Token::Word("w".to_string()), Token::DQuote]
        );
    }

    fn lex(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let options = Rc::new(RefCell::new(Options::default()));
        let mut lexer = Lexer {
            cursor: Cursor::new(
                InputLines::Single(Some(String::from(input))),
                false,
                options.clone(),
            ),
            options: options.clone(),
        };

        loop {
            let token = lexer.next_token(Mode::InDoubleQuotes);
            if token == Token::EOF {
                break;
            }
            tokens.push(token);
        }

        tokens
    }
}
