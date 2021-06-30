use crate::cursor::{Cursor, EOF_CHAR};
use crate::token::Token;

/// Returns the next [`Token`] in single-quoted mode.
pub(crate) fn next_single_quoted_token(cursor: &mut Cursor) -> Token {
    match *cursor.peek() {
        EOF_CHAR => Token::EOF, // End of input.
        '\'' => {
            cursor.next();
            Token::SQuote
        }
        _ => Token::Word(cursor.read_until(|ch| ch == &'\'')),
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
            "\"", "|", "(", ")", "<", ">", "&", ";", "&&", "||", ";;", "<<", ">>", "<&", ">&", "<>",
            "<<-", ">|",
        ];

        for input in inputs {
            // Should be considered words in single-quoted mode.
            assert_eq!(lex(input), vec![Token::Word(String::from(input))]);
        }
    }

    #[test]
    fn it_identifies_squote_tokens() {
        assert_eq!(lex("'"), vec![Token::SQuote]);
        assert_eq!(lex("w'"), vec![Token::Word("w".to_string()), Token::SQuote]);
    }

    fn lex(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut lexer = Lexer {
            cursor: Cursor::new(InputLines::Single(Some(String::from(input)))),
        };

        loop {
            let token = lexer.next_token(Mode::InSingleQuotes);
            if token == Token::EOF {
                break;
            }
            tokens.push(token);
        }

        tokens
    }
}
