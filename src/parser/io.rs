use crate::{
    ast::{IoFile, IoHere, IoRedirect, Word},
    token::Token,
};

use super::{adapter::LexerAdapter, error::ParseError, word::WordParser, Parse};

/// Parses [`crate::ast::IoFile`] syntax.
///```yacc
/// io_file          : '<'       filename
///                  | LESSAND   filename
///                  | '>'       filename
///                  | GREATAND  filename
///                  | DGREAT    filename
///                  | LESSGREAT filename
///                  | CLOBBER   filename
///                  ;
///```
pub(crate) struct IoFileParser {
    word_parser: WordParser,
}

impl IoFileParser {
    pub fn new(word_parser: WordParser) -> Self {
        Self { word_parser }
    }

    fn filename(&mut self, lexer: &mut LexerAdapter) -> Result<Word, ParseError> {
        self.word_parser.parse(lexer)
    }
}

impl Parse for IoFileParser {
    type Item = IoFile;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        match lexer.peek_token() {
            Token::Less => {
                lexer.next_token();
                self.filename(lexer).map(|file| IoFile::Less(file))
            }
            Token::LessAnd => {
                lexer.next_token();
                self.filename(lexer).map(|file| IoFile::LessAnd(file))
            }
            Token::Great => {
                lexer.next_token();
                self.filename(lexer).map(|file| IoFile::Great(file))
            }
            Token::GreatAnd => {
                lexer.next_token();
                self.filename(lexer).map(|file| IoFile::GreatAnd(file))
            }
            Token::DGreat => {
                lexer.next_token();
                self.filename(lexer).map(|file| IoFile::DGreat(file))
            }
            Token::LessGreat => {
                lexer.next_token();
                self.filename(lexer).map(|file| IoFile::LessGreat(file))
            }
            Token::Clobber => {
                lexer.next_token();
                self.filename(lexer).map(|file| IoFile::Clobber(file))
            }
            _ => Err(ParseError::UnexpectedToken(lexer.peek_token().clone())),
        }
    }
}

/// Parses [`crate::ast::IoHere`] syntax.
///```yacc
/// io_here          : DLESS     here_end
///                  | DLESSDASH here_end
///                  ;
///```
pub(crate) struct IoHereParser {
    word_parser: WordParser,
}

impl IoHereParser {
    pub fn new(word_parser: WordParser) -> Self {
        Self { word_parser }
    }

    fn here_end(&mut self, lexer: &mut LexerAdapter) -> Result<Word, ParseError> {
        self.word_parser.parse(lexer)
    }
}

impl Parse for IoHereParser {
    type Item = IoHere;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        match lexer.peek_token() {
            Token::DLess => {
                lexer.next_token();
                self.here_end(lexer).map(|end| IoHere::DLess(end))
            }
            Token::DLessDash => {
                lexer.next_token();
                self.here_end(lexer).map(|end| IoHere::DLessDash(end))
            }
            _ => Err(ParseError::UnexpectedToken(lexer.peek_token().clone())),
        }
    }
}

/// Parses [`crate::ast::IoRedirect`] syntax.
///```yacc
/// io_redirect      :           io_file
///                  | IO_NUMBER io_file
///                  |           io_here
///                  | IO_NUMBER io_here
///                  ;
///```
pub(crate) struct IoRedirectParser {
    io_file_parser: IoFileParser,
    io_here_parser: IoHereParser,
}

impl IoRedirectParser {
    pub fn new(io_file_parser: IoFileParser, io_here_parser: IoHereParser) -> Self {
        Self {
            io_file_parser,
            io_here_parser,
        }
    }
}

impl Parse for IoRedirectParser {
    type Item = IoRedirect;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut io_number: Option<u8> = None;
        if let Token::IoNumber(number) = lexer.peek_token() {
            io_number = Some(*number);
            lexer.next_token();
        }

        if let Ok(io_file) = self.io_file_parser.parse(lexer) {
            Ok(IoRedirect::IoFile(io_number, io_file))
        } else if let Ok(io_here) = self.io_here_parser.parse(lexer) {
            Ok(IoRedirect::IoHere(io_number, io_here))
        } else {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        }
    }
}
