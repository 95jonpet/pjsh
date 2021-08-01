use crate::{ast::SeparatorOp, token::Token};

use super::{adapter::LexerAdapter, error::ParseError, Parse};

/// Parses linebreaks.
///```yacc
/// linebreak        : newline_list
///                  | /* empty */
///                  ;
///```
pub(crate) struct LinebreakParser {
    newline_list_parser: NewlineListParser,
}

impl LinebreakParser {
    pub fn new(newline_list_parser: NewlineListParser) -> Self {
        Self {
            newline_list_parser,
        }
    }
}

impl Parse for LinebreakParser {
    type Item = ();

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        self.newline_list_parser.parse(lexer).or(Ok(()))
    }
}

/// Parses newline lists.
///```yacc
/// newline_list     :              NEWLINE
///                  | newline_list NEWLINE
///                  ;
///```
pub(crate) struct NewlineListParser {}

impl NewlineListParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parse for NewlineListParser {
    type Item = ();

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut newline_seen = false;

        while let Token::Newline = lexer.peek_token() {
            lexer.next_token();
            lexer.advance_line();
            newline_seen = true;
        }

        if newline_seen {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        }
    }
}

/// Parses separators.
///```yacc
/// separator        : separator_op linebreak
///                  | newline_list
///                  ;
///```
pub(crate) struct SeparatorParser {
    separator_op_parser: SeparatorOpParser,
    linebreak_parser: LinebreakParser,
    newline_list_parser: NewlineListParser,
}

impl Parse for SeparatorParser {
    type Item = ();

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        match self.separator_op_parser.parse(lexer) {
            Ok(_) => self.linebreak_parser.parse(lexer),
            _ => self.newline_list_parser.parse(lexer),
        }
    }
}

/// Parses [`crate::ast::SeparatorOp`] syntax.
///```yacc
/// separator_op     : '&'
///                  | ';'
///                  ;
///```
pub(crate) struct SeparatorOpParser {}

impl SeparatorOpParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parse for SeparatorOpParser {
    type Item = SeparatorOp;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        match lexer.peek_token() {
            Token::And => {
                lexer.next_token();
                Ok(SeparatorOp::Async)
            }
            Token::Semi => {
                lexer.next_token();
                Ok(SeparatorOp::Serial)
            }
            _ => Err(ParseError::UnexpectedToken(lexer.peek_token().clone())),
        }
    }
}
