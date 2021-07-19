use crate::{
    ast::{Word, Wordlist},
    token::Token,
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
                if let Token::Word(word) = lexer.next_token() {
                    return Ok(Word(word));
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

#[cfg(test)]
mod tests {
    use crate::lexer::{Lex, Mode};

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
        let tokens = vec![Token::Word(String::from("word"))];
        let mut word_parser = WordParser::new();
        assert_eq!(
            word_parser.parse(&mut adapter(tokens)),
            Ok(Word(String::from("word"))),
        );
    }

    #[test]
    fn it_parses_wordlists() {
        let tokens = vec![
            Token::Word(String::from("word1")),
            Token::Word(String::from("word2")),
        ];
        let mut wordlist_parser = WordlistParser::new(WordParser::new());
        assert_eq!(
            wordlist_parser.parse(&mut adapter(tokens)),
            Ok(Wordlist(vec![
                Word(String::from("word1")),
                Word(String::from("word2")),
            ])),
        );
    }
}
