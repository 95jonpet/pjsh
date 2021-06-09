use crate::shell::{self, Shell};
use crate::token::{Keyword, Operator, Separator, Token};
use std::cell::RefCell;
use std::iter::Peekable;
use std::rc::Rc;
use std::vec::IntoIter;

const UNEXPECTED_EOF: &str = "Unexpected EOF";

pub struct Lexer {
    #[allow(dead_code)]
    shell: Rc<RefCell<Shell>>,
    line: Peekable<IntoIter<char>>,
    ifs: String,
}

impl Lexer {
    pub fn new(line: &str, shell: Rc<RefCell<Shell>>) -> Self {
        Self {
            shell,
            line: line.chars().collect::<Vec<_>>().into_iter().peekable(),
            ifs: String::from(" \t\n"),
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

    /// Advances the character iterator and returns the next line of input.
    fn next_line(&mut self) -> Result<(), String> {
        if let Some(s) = self.shell.borrow_mut().next_prompt(shell::PS2) {
            self.line = s.chars().collect::<Vec<_>>().into_iter().peekable();
            Ok(())
        } else {
            Err(String::from("expected more input but found one"))
        }
    }

    /// Advances the character iterator while a predicate holds and and returns a string containing
    /// all characters that are visited in the process.
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

    /// Returns a token for a specified keyword.
    ///
    /// # Arguments
    ///
    /// * `word` - A word that may or may not be a keyword.
    ///
    /// Returns [`None`] if the specified word is not a keyword.
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

    /// Returns the next token, advancing the character iterator in the process.
    pub fn next_token(&mut self) -> Option<Token> {
        let ifs = self.ifs.clone();

        self.next_while(|c| ifs.contains(*c)); // Skip all whitespace.
        self.next_while(|c| *c == '\r' || *c == '\n'); // Skip newline characters.
        match self.peek_char() {
            Some('#') => {
                let comment = self.next_while(|c| c != &'\n');
                Some(Token::Comment(comment))
            }
            Some('=') => {
                self.next_char(); // Skip peeked char.
                Some(Token::Operator(Operator::Equal))
            }
            Some('!') => {
                self.next_char(); // Skip peeked char.
                Some(Token::Operator(Operator::Bang))
            }
            Some('|') => {
                self.next_char(); // Skip peeked char.
                if let Some('|') = self.peek_char() {
                    self.next_char(); // Skip peeked char.
                    Some(Token::Operator(Operator::Or))
                } else {
                    Some(Token::Operator(Operator::Pipe))
                }
            }
            Some('&') => {
                self.next_char(); // Skip peeked char.
                if let Some('&') = self.peek_char() {
                    self.next_char(); // Skip peeked char.
                    Some(Token::Operator(Operator::And))
                } else {
                    Some(Token::Operator(Operator::Ampersand))
                }
            }
            Some(';') => {
                self.next_char(); // Skip peeked char.
                Some(Token::Separator(Separator::Semicolon))
            }
            Some('\'') => {
                self.next_char(); // Skip delimiter.

                let mut string_content = String::new();
                loop {
                    match self.peek_char() {
                        Some('\\') => {
                            self.next_char(); // Skip backslash.

                            // Add escaped token to the string.
                            match self.peek_char() {
                                Some('\'') => string_content.push(self.next_char().unwrap()),
                                Some(_) => {
                                    // Non-escapable character.
                                    string_content.push('\\'); // Add backslash.
                                    string_content.push(self.next_char().unwrap());
                                }
                                _ => (),
                            }
                        }
                        Some('\'') => {
                            self.next_char(); // Skip delimiter.
                            return Some(Token::Word(string_content));
                        }
                        Some(_) => {
                            string_content.push(self.next_char().unwrap());
                        }
                        None => (),
                    }
                }
            }
            Some(current_char) if current_char.is_ascii_alphanumeric() => {
                let word = self.next_while(|c| c.is_ascii_alphanumeric() || c == &'_' || c == &'.');

                if let Some(token) = Self::keyword_token(&word) {
                    return Some(token);
                }

                if let Some(next_char) = self.peek_char() {
                    if next_char == &'=' {
                        self.next_char(); // Skip peeked char.

                        // Right hand of assignment is none.
                        let token = match self.peek_char() {
                            Some(next_char) if ifs.contains(*next_char) => {
                                self.next_char(); // Skip peeked char.
                                Token::Assign(word, String::new())
                            }
                            Some('\'') => {
                                self.next_char();
                                let string = self.next_while(|c| *c != '\'');
                                self.next_char();
                                Token::Assign(word, string)
                            }
                            Some(';') | None => {
                                self.next_char(); // Skip peeked char.
                                Token::Assign(word, String::new())
                            }
                            _ => {
                                let value = self.next_while(|c| {
                                    c.is_ascii_alphanumeric() || c == &'_' || c == &'.'
                                });
                                Token::Assign(word, value)
                            }
                        };
                        return Some(token);
                    }
                }

                Some(Token::Word(word))
            }
            Some(_) => {
                // Treat unknown lexemes as string literals.
                let string_content = self.next_while(|c| !ifs.contains(*c));
                Some(Token::Word(string_content))
            }
            _ => None,
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_identifies_comments() {
        assert_eq!(
            tokenize("#This is a comment."),
            vec![Token::Comment(String::from("#This is a comment."))]
        );
    }

    #[test]
    fn it_identifies_strings() {
        assert_eq!(
            tokenize("'This is a string'"),
            vec![Token::Word(String::from("This is a string")),]
        );
    }

    #[test]
    fn it_identifies_strings_with_escaped_chars() {
        assert_eq!(
            tokenize("'It\\'s a string'"),
            vec![Token::Word(String::from("It's a string")),]
        );
        assert_eq!(
            tokenize("'\\n'"), // Should not be escaped.
            vec![Token::Word(String::from("\\n")),]
        );
    }

    #[test]
    fn it_identifies_separator_semicolon() {
        assert_eq!(tokenize(";"), vec![Token::Separator(Separator::Semicolon)]);
    }

    #[test]
    fn it_identifies_words() {
        assert_eq!(
            tokenize("lowercase"),
            vec![Token::Word(String::from("lowercase"))]
        );
        assert_eq!(
            tokenize("UPPERCASE"),
            vec![Token::Word(String::from("UPPERCASE"))]
        );
        assert_eq!(
            tokenize("MixedCase"),
            vec![Token::Word(String::from("MixedCase"))]
        );
        assert_eq!(
            tokenize("with_underscore"),
            vec![Token::Word(String::from("with_underscore"))]
        );
        assert_eq!(
            tokenize("number123"),
            vec![Token::Word(String::from("number123"))]
        );
        assert_eq!(
            tokenize("two words"),
            vec![
                Token::Word(String::from("two")),
                Token::Word(String::from("words"))
            ]
        );
        assert_eq!(
            tokenize("cat file.extension"),
            vec![
                Token::Word(String::from("cat")),
                Token::Word(String::from("file.extension"))
            ]
        );
    }

    #[test]
    fn it_ignores_newline_chars() {
        assert_eq!(tokenize("\r\n"), vec![]);
    }

    #[test]
    fn it_identifies_operator_ampersand() {
        assert_eq!(tokenize("&"), vec![Token::Operator(Operator::Ampersand),]);
        assert_eq!(
            tokenize("code &"),
            vec![
                Token::Word(String::from("code")),
                Token::Operator(Operator::Ampersand),
            ]
        );
    }

    #[test]
    fn it_identifies_operator_and() {
        assert_eq!(tokenize("&&"), vec![Token::Operator(Operator::And),]);
        assert_eq!(
            tokenize("x && y"),
            vec![
                Token::Word(String::from("x")),
                Token::Operator(Operator::And),
                Token::Word(String::from("y")),
            ]
        );
    }

    #[test]
    fn it_identifies_assignments() {
        assert_eq!(
            tokenize("x=1234"),
            vec![Token::Assign(String::from("x"), String::from("1234"))]
        );
        assert_eq!(
            tokenize("x= test"),
            vec![
                Token::Assign(String::from("x"), String::new()),
                Token::Word(String::from("test")),
            ]
        );
        assert_eq!(
            tokenize("x="),
            vec![Token::Assign(String::from("x"), String::new()),]
        );
        assert_eq!(
            tokenize("x=;"),
            vec![Token::Assign(String::from("x"), String::new()),]
        );
        assert_eq!(
            tokenize("run_tests --env=production"),
            vec![
                Token::Word(String::from("run_tests")),
                Token::Word(String::from("--env=production")),
            ]
        );
    }

    #[test]
    fn it_identifies_operator_bang() {
        assert_eq!(tokenize("!"), vec![Token::Operator(Operator::Bang)]);
        assert_eq!(
            tokenize("! true"),
            vec![
                Token::Operator(Operator::Bang),
                Token::Word(String::from("true"))
            ]
        );
    }

    #[test]
    fn it_identifies_operator_equal() {
        assert_eq!(tokenize("="), vec![Token::Operator(Operator::Equal)]);
        assert_eq!(
            tokenize("x = 1234"),
            vec![
                Token::Word(String::from("x")),
                Token::Operator(Operator::Equal),
                Token::Word(String::from("1234"))
            ]
        );
    }

    #[test]
    fn it_identifies_operator_or() {
        assert_eq!(tokenize("||"), vec![Token::Operator(Operator::Or),]);
        assert_eq!(
            tokenize("x || y"),
            vec![
                Token::Word(String::from("x")),
                Token::Operator(Operator::Or),
                Token::Word(String::from("y")),
            ]
        );
    }

    #[test]
    fn it_identifies_operator_pipe() {
        assert_eq!(tokenize("|"), vec![Token::Operator(Operator::Pipe)]);
        assert_eq!(
            tokenize("cat file_name | grep value"),
            vec![
                Token::Word(String::from("cat")),
                Token::Word(String::from("file_name")),
                Token::Operator(Operator::Pipe),
                Token::Word(String::from("grep")),
                Token::Word(String::from("value")),
            ]
        );
    }

    /// Tokenizes a string using a [`Lexer`].
    fn tokenize(input: &str) -> Vec<super::Token> {
        let mut lexer = Lexer::new(
            input,
            Rc::new(RefCell::new(Shell::from_command(String::from("")))),
        );
        let mut tokens: Vec<super::Token> = Vec::new();

        while let Some(token) = lexer.next_token() {
            tokens.push(token);
        }

        tokens
    }
}
