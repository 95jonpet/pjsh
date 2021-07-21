use crate::{
    ast::{AssignmentWord, Word, Wordlist},
    token::{Token, Unit},
};

use super::{adapter::LexerAdapter, error::ParseError, Parse};

pub(crate) struct WordParser {}

impl WordParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parse for WordParser {
    type Item = Word;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        match lexer.peek_token() {
            Token::Word(_) => {
                if let Token::Word(units) = lexer.next_token() {
                    return Ok(Word(units));
                }
                unreachable!()
            }
            token => Err(ParseError::UnexpectedToken(token.clone())),
        }
    }
}

pub(crate) struct WordlistParser {
    word_parser: WordParser,
}

impl WordlistParser {
    pub fn new(word_parser: WordParser) -> Self {
        Self { word_parser }
    }
}

impl Parse for WordlistParser {
    type Item = Wordlist;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut words = Vec::new();
        while let Ok(word) = self.word_parser.parse(lexer) {
            words.push(word);
        }

        if words.is_empty() {
            return Err(ParseError::UnexpectedToken(lexer.peek_token().clone()));
        }

        Ok(Wordlist(words))
    }
}

pub(crate) struct AssignmentWordParser {}

impl AssignmentWordParser {
    pub fn new() -> Self {
        Self {}
    }

    fn is_assignment(units: &Vec<Unit>) -> bool {
        units.iter().any(|unit| match unit {
            Unit::Literal(literal) => literal.contains("="),
            _ => false,
        })
    }
}

impl Parse for AssignmentWordParser {
    type Item = AssignmentWord;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        match lexer.peek_token() {
            Token::Word(units) if Self::is_assignment(units) => match units.first() {
                Some(Unit::Literal(literal)) => {
                    let split_index = literal.find('=').unwrap();
                    let key = String::from(&literal[..split_index]);
                    let value = String::from(&literal[(split_index + 1)..]);
                    Ok(AssignmentWord(key, value))
                }
                _ => unimplemented!(),
            },
            token => Err(ParseError::UnexpectedToken(token.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lex, Mode},
        token::Unit,
    };

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

        fn advance_line(&mut self) {}
    }

    fn adapter(tokens: Vec<Token>) -> LexerAdapter {
        let lexer = MockLexer::new(tokens);
        LexerAdapter::new(Box::new(lexer))
    }

    #[test]
    fn it_parses_words() {
        let tokens = vec![Token::Word(vec![Unit::Literal(String::from("word"))])];
        let mut word_parser = WordParser::new();
        assert_eq!(
            word_parser.parse(&mut adapter(tokens)),
            Ok(Word(vec![Unit::Literal(String::from("word"))])),
        );
    }

    #[test]
    fn it_parses_wordlists() {
        let tokens = vec![
            Token::Word(vec![Unit::Literal(String::from("word1"))]),
            Token::Word(vec![Unit::Literal(String::from("word2"))]),
        ];
        let mut wordlist_parser = WordlistParser::new(WordParser::new());
        assert_eq!(
            wordlist_parser.parse(&mut adapter(tokens)),
            Ok(Wordlist(vec![
                Word(vec![Unit::Literal(String::from("word1"))]),
                Word(vec![Unit::Literal(String::from("word2"))]),
            ])),
        );
    }
}
