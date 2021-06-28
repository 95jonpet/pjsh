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
