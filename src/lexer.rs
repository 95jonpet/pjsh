use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::rc::Rc;
use std::vec::IntoIter;

const UNEXPECTED_EOF: &str = "Unexpected EOF";

#[derive(Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Keyword(Keyword),
    Separator(Separator),
    Operator(Operator),
    Literal(Literal),
    Comment(String),
}

#[derive(Debug, PartialEq)]
pub enum Separator {
    Semicolon,
    SingleQuote,
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
    String(String),
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Case,
    Do,
    Done,
    Elif,
    Else,
    Esac,
    Fi,
    For,
    If,
    In,
    Then,
    Until,
    While,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Assign,
    Equal,
    Pipe,
}

pub struct Lexer {
    // shell: Rc<RefCell<Shell>>,
    line: Peekable<IntoIter<char>>,
    queued_tokens: VecDeque<Token>,
}

impl Lexer {
    pub fn new(line: &str /*, shell: Rc<RefCell<Shell>>*/) -> Self {
        Self {
            // shell,
            line: line.chars().collect::<Vec<_>>().into_iter().peekable(),
            queued_tokens: VecDeque::new(),
        }
    }

    /// Returns a reference to the `next_char()` value without advancing the character iterator.
    fn peek_char(&mut self) -> Option<&char> {
        self.line.peek()
    }

    /// Advances the character iterator and returns the next value.
    fn next_char(&mut self) -> Option<char> {
        self.line.next()
    }

    /// Advances the character iterator while a predicate holds and and returns a string containing all characters that are visited in the process.
    fn next_while<P>(&mut self, predicate: P) -> String
    where
        P: Fn(&char) -> bool,
    {
        let mut result = String::new();
        while let Some(character) = self.peek_char() {
            if !predicate(character) {
                break;
            }

            result.push(self.next_char().expect(UNEXPECTED_EOF));
        }
        result
    }

    /// Enqueues a token that must be returned the next time a token is requested.
    fn enqueue_token(&mut self, token: Token) {
        self.queued_tokens.push_back(token)
    }

    /// Skips all whitespace characters.
    fn skip_whitespace(&mut self) {
        let mut next = self.peek_char();
        while next.is_some() && next.unwrap().is_whitespace() {
            self.next_char();
            next = self.peek_char();
        }
    }

    /// Returns a token for a specified keyword.
    ///
    /// Returns `None` if the specified word is not a keyword.
    fn keyword_token(word: &String) -> Option<Token> {
        match word.as_str() {
            "case" => Some(Token::Keyword(Keyword::Case)),
            "do" => Some(Token::Keyword(Keyword::Do)),
            "done" => Some(Token::Keyword(Keyword::Done)),
            "elif" => Some(Token::Keyword(Keyword::Elif)),
            "else" => Some(Token::Keyword(Keyword::Else)),
            "esac" => Some(Token::Keyword(Keyword::Esac)),
            "fi" => Some(Token::Keyword(Keyword::Fi)),
            "for" => Some(Token::Keyword(Keyword::For)),
            "if" => Some(Token::Keyword(Keyword::If)),
            "in" => Some(Token::Keyword(Keyword::In)),
            "then" => Some(Token::Keyword(Keyword::Then)),
            "until" => Some(Token::Keyword(Keyword::Until)),
            "while" => Some(Token::Keyword(Keyword::While)),
            _ => None,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        if !self.queued_tokens.is_empty() {
            return self.queued_tokens.pop_front();
        }

        self.skip_whitespace();
        match self.peek_char() {
            Some('#') => {
                let comment = self.next_while(|c| c != &'\n');
                Some(Token::Comment(comment))
            }
            Some('=') => {
                self.next_char(); // Skip peeked char.
                Some(Token::Operator(Operator::Equal))
            }
            Some('|') => {
                self.next_char(); // Skip peeked char.
                Some(Token::Operator(Operator::Pipe))
            }
            Some(';') => {
                self.next_char(); // Skip peeked char.
                Some(Token::Separator(Separator::Semicolon))
            }
            Some('\'') => {
                // Enqueue the token for the first delimiter.
                self.enqueue_token(Token::Separator(Separator::SingleQuote));
                self.next_char();

                // Enqueue a token for the string content.
                let string_content = self.next_while(|c| c != &'\'');
                self.enqueue_token(Token::Literal(Literal::String(string_content)));

                // Enqueue the token for the last delimiter.
                self.enqueue_token(Token::Separator(Separator::SingleQuote));
                self.next_char();

                // Return the first delimiter.
                // The other tokens will be returned in subsequent calls.
                self.queued_tokens.pop_front()
            }
            Some(current_char) if current_char.is_digit(10) => {
                let digits = self.next_while(|c| c.is_digit(10));
                digits
                    .parse::<i64>()
                    .map_or(None, |num| Some(Token::Literal(Literal::Integer(num))))
            }
            Some(current_char) if current_char.is_ascii_alphanumeric() => {
                let word = self.next_while(|c| c.is_ascii_alphanumeric() || c == &'_');

                if let Some(token) = Self::keyword_token(&word) {
                    return Some(token);
                }

                if let Some(next_char) = self.peek_char() {
                    if next_char == &'=' {
                        self.next_char(); // Skip peeked char.
                        self.enqueue_token(Token::Operator(Operator::Assign));

                        // Right hand of assignment is none.
                        match self.peek_char() {
                            Some(next_char) if next_char.is_whitespace() => {
                                self.next_char(); // Skip peeked char.
                                self.enqueue_token(Token::Literal(Literal::String(String::new())));
                            }
                            Some(';') | None => {
                                self.next_char(); // Skip peeked char.
                                self.enqueue_token(Token::Literal(Literal::String(String::new())));
                            }
                            _ => (),
                        }
                    }
                }

                Some(Token::Identifier(word))
            }
            Some(_) => {
                // Treat unknown lexemes as string literals.
                let string_content = self.next_while(|c| !c.is_whitespace());
                Some(Token::Literal(Literal::String(string_content)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_identifies_comments() {
        assert_eq!(
            tokens("#This is a comment."),
            vec![Token::Comment(String::from("#This is a comment."))]
        );
    }

    #[test]
    fn it_identifies_integers() {
        assert_eq!(tokens("0"), vec![Token::Literal(Literal::Integer(0))]);
        assert_eq!(tokens("1"), vec![Token::Literal(Literal::Integer(1))]);
        assert_eq!(tokens("57"), vec![Token::Literal(Literal::Integer(57))]);
        assert_eq!(tokens("100"), vec![Token::Literal(Literal::Integer(100))]);
    }

    #[test]
    fn it_identifies_strings() {
        assert_eq!(
            tokens("'This is a string'"),
            vec![
                Token::Separator(Separator::SingleQuote),
                Token::Literal(Literal::String(String::from("This is a string"))),
                Token::Separator(Separator::SingleQuote),
            ]
        );
    }

    #[test]
    fn it_identifies_separator_semicolon() {
        assert_eq!(tokens(";"), vec![Token::Separator(Separator::Semicolon)]);
    }

    #[test]
    fn it_identifies_identifiers() {
        assert_eq!(
            tokens("lowercase"),
            vec![Token::Identifier(String::from("lowercase"))]
        );
        assert_eq!(
            tokens("UPPERCASE"),
            vec![Token::Identifier(String::from("UPPERCASE"))]
        );
        assert_eq!(
            tokens("MixedCase"),
            vec![Token::Identifier(String::from("MixedCase"))]
        );
        assert_eq!(
            tokens("with_underscore"),
            vec![Token::Identifier(String::from("with_underscore"))]
        );
        assert_eq!(
            tokens("number123"),
            vec![Token::Identifier(String::from("number123"))]
        );
    }

    #[test]
    fn it_identifies_operator_assign() {
        assert_eq!(
            tokens("x=1234"),
            vec![
                Token::Identifier(String::from("x")),
                Token::Operator(Operator::Assign),
                Token::Literal(Literal::Integer(1234))
            ]
        );
        assert_eq!(
            tokens("x= test"),
            vec![
                Token::Identifier(String::from("x")),
                Token::Operator(Operator::Assign),
                Token::Literal(Literal::String(String::new())),
                Token::Identifier(String::from("test")),
            ]
        );
        assert_eq!(
            tokens("x="),
            vec![
                Token::Identifier(String::from("x")),
                Token::Operator(Operator::Assign),
                Token::Literal(Literal::String(String::new())),
            ]
        );
        assert_eq!(
            tokens("x=;"),
            vec![
                Token::Identifier(String::from("x")),
                Token::Operator(Operator::Assign),
                Token::Literal(Literal::String(String::new())),
            ]
        );
    }

    #[test]
    fn it_identifies_operator_equal() {
        assert_eq!(tokens("="), vec![Token::Operator(Operator::Equal)]);
        assert_eq!(
            tokens("x = 1234"),
            vec![
                Token::Identifier(String::from("x")),
                Token::Operator(Operator::Equal),
                Token::Literal(Literal::Integer(1234))
            ]
        );
    }

    #[test]
    fn it_identifies_operator_pipe() {
        assert_eq!(tokens("|"), vec![Token::Operator(Operator::Pipe)]);
        assert_eq!(
            tokens("cat file_name | grep value"),
            vec![
                Token::Identifier(String::from("cat")),
                Token::Identifier(String::from("file_name")),
                Token::Operator(Operator::Pipe),
                Token::Identifier(String::from("grep")),
                Token::Identifier(String::from("value")),
            ]
        );
    }

    fn tokens(input: &str) -> Vec<super::Token> {
        let mut lexer = Lexer::new(input);
        let mut tokens: Vec<super::Token> = Vec::new();

        while let Some(token) = lexer.next_token() {
            tokens.push(token);
        }

        tokens
    }
}
