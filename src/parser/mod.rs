use crate::{
    lexer::{Lexer, Mode},
    token::Token,
};

#[derive(Debug, PartialEq)]
enum ParseError {
    UnexpectedCharSequence,
    UnexpectedToken,
}

// enum Result {
//     NonTerminal(Token),
//     Terminal(Token),
// }

struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer }
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        match self.lexer.next_token(Mode::Unquoted) {
            Token::Newline => self.newline_list(),
            Token::EOF => Ok(()),
            Token::Unknown => Err(ParseError::UnexpectedCharSequence),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    fn cmd_name(&mut self) -> Result<(), ParseError> {
        self.eat_word()
    }

    fn cmd_word(&mut self) -> Result<(), ParseError> {
        self.eat_word()
    }

    fn here_end(&mut self) -> Result<(), ParseError> {
        self.eat_word()
    }

    fn filename(&mut self) -> Result<(), ParseError> {
        self.eat_word()
    }

    // io_here          : DLESS     here_end
    //                  | DLESSDASH here_end
    //                  ;
    fn io_here(&mut self) -> Result<(), ParseError> {
        match self.lexer.next_token(Mode::Unquoted) {
            Token::DLess | Token::DLessDash => self.here_end(),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    // io_file          : '<'       filename
    //                  | LESSAND   filename
    //                  | '>'       filename
    //                  | GREATAND  filename
    //                  | DGREAT    filename
    //                  | LESSGREAT filename
    //                  | CLOBBER   filename
    //                  ;
    fn io_file(&mut self) -> Result<(), ParseError> {
        match self.lexer.next_token(Mode::Unquoted) {
            Token::Less
            | Token::LessAnd
            | Token::Great
            | Token::GreatAnd
            | Token::DGreat
            | Token::LessGreat
            | Token::Clobber => self.filename(),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    // newline_list     :              NEWLINE
    //                  | newline_list NEWLINE
    //                  ;
    fn newline_list(&mut self) -> Result<(), ParseError> {
        match self.lexer.next_token(Mode::Unquoted) {
            Token::Newline => self.newline_list(),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    // linebreak        : newline_list
    //                  | /* empty */
    //                  ;
    fn linebreak(&mut self) -> Result<(), ParseError> {
        match self.lexer.next_token(Mode::Unquoted) {
            Token::Newline => self.newline_list(),
            Token::EOF => Ok(()),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    /// Consumes a word token.
    /// Returns a [`ParseError`] if the next token sequence cannot be parsed into a word.
    fn eat_word(&mut self) -> Result<(), ParseError> {
        match self.lexer.next_token(Mode::Unquoted) {
            Token::Word(_) => Ok(()),
            Token::SQuote => match self.lexer.next_token(Mode::InSingleQuotes) {
                Token::Word(_) => match self.lexer.next_token(Mode::InSingleQuotes) {
                    Token::SQuote => Ok(()),
                    _ => Err(ParseError::UnexpectedToken),
                },
                _ => Err(ParseError::UnexpectedToken),
            },
            _ => Err(ParseError::UnexpectedToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cursor::Cursor, input::InputLines};

    use super::*;

    #[test]
    fn it_parses_newline_list() {
        let input = "\n\n";

        let cursor = Cursor::new(InputLines::Single(Some(String::from(input))));
        let lexer = Lexer::new(cursor);
        let mut parser = Parser::new(lexer);
        let result = parser.parse();

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn it_parses_io_file() {
        let input = "< word";

        let cursor = Cursor::new(InputLines::Single(Some(String::from(input))));
        let lexer = Lexer::new(cursor);
        let mut parser = Parser::new(lexer);
        let result = parser.io_file();

        assert_eq!(result, Ok(()));
    }
}
