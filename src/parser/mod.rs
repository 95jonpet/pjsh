use std::collections::VecDeque;

use crate::{
    lexer::{Lex, Mode},
    token::Token,
};

#[derive(Debug, PartialEq)]
enum ParseError {
    UnexpectedCharSequence,
    UnexpectedToken(Token),
}

// enum Result {
//     NonTerminal(Token),
//     Terminal(Token),
// }

#[derive(Debug, PartialEq)]
struct Word(String);
#[derive(Debug, PartialEq)]
struct Wordlist(Vec<Word>);
#[derive(Debug, PartialEq)]
enum IoFile {
    Less(String),
    LessAnd(String),
    Great(String),
    GreatAnd(String),
    DGreat(String),
    LessGreat(String),
    Clobber(String),
}

struct Parser {
    lexer: Box<dyn Lex>,
    lexer_mode_stack: Vec<Mode>,
    cached_tokens: VecDeque<Token>,
}

const DEFAULT_LEXER_MODE_STACK_CAPACITY: usize = 10;

impl Parser {
    pub fn new(lexer: Box<dyn Lex>) -> Self {
        let mut lexer_mode_stack = Vec::with_capacity(DEFAULT_LEXER_MODE_STACK_CAPACITY);
        lexer_mode_stack.push(Mode::Unquoted);

        Self {
            lexer,
            lexer_mode_stack,
            cached_tokens: VecDeque::new(),
        }
    }

    fn lexer_mode(&self) -> Mode {
        *self
            .lexer_mode_stack
            .first()
            .expect("a lexer mode to be set")
    }

    fn peek_token(&mut self) -> &Token {
        if self.cached_tokens.is_empty() {
            let next_token = self.lexer.next_token(self.lexer_mode());
            self.cached_tokens.push_back(next_token);
        }

        self.cached_tokens.front().unwrap_or(&Token::EOF)
    }

    fn next_token(&mut self) -> Token {
        self.cached_tokens
            .pop_front()
            .unwrap_or_else(|| self.lexer.next_token(self.lexer_mode()))
    }

    fn push_lexer_mode(&mut self, lexer_mode: Mode) {
        if lexer_mode != self.lexer_mode() && !self.cached_tokens.is_empty() {
            unreachable!("The lexer mode should not be changed while peeked tokens are held!");
        }

        self.lexer_mode_stack.push(lexer_mode);
    }

    fn pop_lexer_mode(&mut self) {
        if self.lexer_mode_stack.is_empty() {
            unreachable!("An empty lexer mode stack should not be popped!");
        }
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        match self.lexer.next_token(Mode::Unquoted) {
            Token::Newline => self.newline_list(),
            Token::EOF => Ok(()),
            Token::Unknown => Err(ParseError::UnexpectedCharSequence),
            _ => Err(ParseError::UnexpectedToken(self.next_token())),
        }
    }

    // name             : NAME                     /* Apply rule 5 */
    //                  ;
    fn name(&mut self) -> Result<Word, ParseError> {
        self.eat_word()
    }

    // wordlist         : wordlist WORD
    //                  |          WORD
    //                  ;
    fn wordlist(&mut self, wordlist: Wordlist) -> Result<Wordlist, ParseError> {
        let Wordlist(mut words) = wordlist;
        match self.eat_word() {
            Ok(word) => {
                words.push(word);
                self.wordlist(Wordlist(words))
            }
            Err(error) if words.is_empty() => Err(error),
            Err(_) => Ok(Wordlist(words)),
        }
    }

    // cmd_name         : WORD                   /* Apply rule 7a */
    //                  ;
    fn cmd_name(&mut self) -> Result<Word, ParseError> {
        self.eat_word()
    }

    // cmd_word         : WORD                   /* Apply rule 7b */
    //                  ;
    fn cmd_word(&mut self) -> Result<Word, ParseError> {
        self.eat_word()
    }

    // io_redirect      :           io_file
    //                  | IO_NUMBER io_file
    //                  |           io_here
    //                  | IO_NUMBER io_here
    //                  ;
    // fn io_redirect(&mut self) -> Result<(), ParseError> {}

    // io_file          : '<'       filename
    //                  | LESSAND   filename
    //                  | '>'       filename
    //                  | GREATAND  filename
    //                  | DGREAT    filename
    //                  | LESSGREAT filename
    //                  | CLOBBER   filename
    //                  ;
    fn io_file(&mut self) -> Result<IoFile, ParseError> {
        match self.peek_token() {
            Token::Less => {
                self.next_token();
                self.filename().map(|Word(file)| IoFile::Less(file))
            }
            Token::LessAnd => {
                self.next_token();
                self.filename().map(|Word(file)| IoFile::LessAnd(file))
            }
            Token::Great => {
                self.next_token();
                self.filename().map(|Word(file)| IoFile::Great(file))
            }
            Token::GreatAnd => {
                self.next_token();
                self.filename().map(|Word(file)| IoFile::GreatAnd(file))
            }
            Token::DGreat => {
                self.next_token();
                self.filename().map(|Word(file)| IoFile::DGreat(file))
            }
            Token::LessGreat => {
                self.next_token();
                self.filename().map(|Word(file)| IoFile::LessGreat(file))
            }
            Token::Clobber => {
                self.next_token();
                self.filename().map(|Word(file)| IoFile::Clobber(file))
            }
            _ => Err(ParseError::UnexpectedToken(self.next_token())),
        }
    }

    // filename         : WORD                      /* Apply rule 2 */
    //                  ;
    fn filename(&mut self) -> Result<Word, ParseError> {
        self.eat_word()
    }

    // io_here          : DLESS     here_end
    //                  | DLESSDASH here_end
    //                  ;
    fn io_here(&mut self) -> Result<(), ParseError> {
        match self.peek_token() {
            Token::DLess | Token::DLessDash => {
                self.next_token();
                self.here_end().map(|definition| ())
            }
            _ => Err(ParseError::UnexpectedToken(self.next_token())),
        }
    }

    // here_end         : WORD                      /* Apply rule 3 */
    //                  ;
    fn here_end(&mut self) -> Result<Word, ParseError> {
        self.eat_word()
    }

    // newline_list     :              NEWLINE
    //                  | newline_list NEWLINE
    //                  ;
    fn newline_list(&mut self) -> Result<(), ParseError> {
        match self.peek_token() {
            Token::Newline => {
                self.next_token();
                self.newline_list()
            }
            _ => Ok(()),
        }
    }

    // linebreak        : newline_list
    //                  | /* empty */
    //                  ;
    fn linebreak(&mut self) -> Result<(), ParseError> {
        match self.peek_token() {
            Token::Newline => {
                self.next_token();
                self.newline_list()
            }
            Token::EOF => {
                self.next_token();
                Ok(())
            }
            _ => Err(ParseError::UnexpectedToken(self.next_token())),
        }
    }

    // separator_op     : '&'
    //                  | ';'
    //                  ;
    fn separator_op(&mut self) -> Result<(), ParseError> {
        match self.peek_token() {
            Token::And | Token::Semi => {
                self.next_token();
                Ok(())
            }
            _ => Err(ParseError::UnexpectedToken(self.next_token())),
        }
    }

    // separator        : separator_op linebreak
    //                  | newline_list
    //                  ;
    fn separator(&mut self) -> Result<(), ParseError> {
        match self.separator_op() {
            Ok(_) => self.linebreak(),
            _ => self.newline_list(),
        }
    }

    /// Consumes a word token.
    /// Returns a [`ParseError`] if the next token sequence cannot be parsed into a word.
    fn eat_word(&mut self) -> Result<Word, ParseError> {
        match self.peek_token() {
            Token::Word(_) => {
                if let Token::Word(word) = self.next_token() {
                    Ok(Word(word))
                } else {
                    unreachable!()
                }
            }
            Token::SQuote => {
                self.next_token();
                self.push_lexer_mode(Mode::InSingleQuotes);
                match self.next_token() {
                    Token::Word(word) => match self.next_token() {
                        Token::SQuote => {
                            self.pop_lexer_mode();
                            Ok(Word(word))
                        }
                        _ => Err(ParseError::UnexpectedToken(self.next_token())),
                    },
                    _ => Err(ParseError::UnexpectedToken(self.next_token())),
                }
            }
            _ => Err(ParseError::UnexpectedToken(self.next_token())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    struct MockLexer {
        tokens: Vec<Token>,
    }

    impl MockLexer {
        fn new(mut tokens: Vec<Token>) -> Self {
            tokens.reverse();
            Self { tokens }
        }
    }

    impl Lex for MockLexer {
        fn next_token(&mut self, _mode: Mode) -> Token {
            self.tokens.pop().unwrap_or(Token::EOF)
        }
    }

    fn parser(tokens: Vec<Token>) -> Parser {
        let lexer = MockLexer::new(tokens);
        let parser = Parser::new(Box::new(lexer));
        parser
    }

    #[test]
    fn it_parses_newline_list() {
        assert_eq!(
            Ok(()),
            parser(vec![Token::Newline, Token::Newline]).newline_list()
        );
    }

    #[test]
    fn it_parses_wordlist() {
        let tokens = vec![
            Token::Word(String::from("first")),
            Token::Word(String::from("second")),
        ];
        assert_eq!(
            Ok(Wordlist(vec![
                Word(String::from("first")),
                Word(String::from("second"))
            ])),
            parser(tokens).wordlist(Wordlist(Vec::new()))
        );
    }

    #[test]
    fn it_parses_io_file() {
        let prefix_tokens = [
            (Token::Less, IoFile::Less(String::from("word"))),
            (Token::LessAnd, IoFile::LessAnd(String::from("word"))),
            (Token::Great, IoFile::Great(String::from("word"))),
            (Token::GreatAnd, IoFile::GreatAnd(String::from("word"))),
            (Token::DGreat, IoFile::DGreat(String::from("word"))),
            (Token::LessGreat, IoFile::LessGreat(String::from("word"))),
            (Token::Clobber, IoFile::Clobber(String::from("word"))),
        ];

        for (prefix, io_file) in prefix_tokens {
            assert_eq!(
                Ok(io_file),
                parser(vec![prefix, Token::Word(String::from("word"))]).io_file()
            );
        }
    }

    #[test]
    fn it_parses_linebreak() {
        assert_eq!(
            Ok(()),
            parser(vec![Token::Newline, Token::Newline]).linebreak()
        );
    }

    #[test]
    fn it_parses_separator_op() {
        for separator in vec![Token::And, Token::Semi] {
            assert_eq!(Ok(()), parser(vec![separator]).separator_op());
        }
    }

    #[test]
    fn it_parses_separator() {
        let token_groups = vec![
            vec![Token::Semi, Token::Newline],
            vec![Token::Newline, Token::Newline],
        ];
        for tokens in token_groups {
            assert_eq!(Ok(()), parser(tokens).separator());
        }
    }
}
