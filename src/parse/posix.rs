use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{CompleteCommand, Program},
    lexer::Lex,
    options::Options,
    token::Token,
};

use super::{
    adapter::LexerAdapter,
    command::{
        CmdPrefixParser, CmdSuffixParser, CommandParser, CompleteCommandParser,
        CompleteCommandsParser, ProgramParser, SimpleCommandParser,
    },
    error::ParseError,
    io::{IoFileParser, IoHereParser, IoRedirectParser},
    meta::{LinebreakParser, NewlineListParser, SeparatorOpParser},
    pipeline::{AndOrParser, ListParser, PipeSequenceParser, PipelineParser},
    word::{AssignmentWordParser, WordParser},
    Parse,
};

pub struct PosixParser {
    complete_command_parser: CompleteCommandParser,
    program_parser: ProgramParser,
    lexer_adapter: LexerAdapter,
    options: Rc<RefCell<Options>>,
}

impl PosixParser {
    pub fn new(lexer: Box<dyn Lex>, options: Rc<RefCell<Options>>) -> Self {
        let complete_command_parser = CompleteCommandParser::new(
            ListParser::new(
                AndOrParser::new(
                    PipelineParser::new(PipeSequenceParser::new(
                        CommandParser::new(SimpleCommandParser::new(
                            WordParser::new(),
                            CmdPrefixParser::new(
                                IoRedirectParser::new(
                                    IoFileParser::new(WordParser::new()),
                                    IoHereParser::new(WordParser::new()),
                                ),
                                AssignmentWordParser::new(),
                            ),
                            CmdSuffixParser::new(
                                WordParser::new(),
                                IoRedirectParser::new(
                                    IoFileParser::new(WordParser::new()),
                                    IoHereParser::new(WordParser::new()),
                                ),
                            ),
                        )),
                        LinebreakParser::new(NewlineListParser::new()),
                    )),
                    LinebreakParser::new(NewlineListParser::new()),
                ),
                SeparatorOpParser::new(),
            ),
            SeparatorOpParser::new(),
        );

        let program_parser = ProgramParser::new(
            CompleteCommandsParser::new(
                CompleteCommandParser::new(
                    ListParser::new(
                        AndOrParser::new(
                            PipelineParser::new(PipeSequenceParser::new(
                                CommandParser::new(SimpleCommandParser::new(
                                    WordParser::new(),
                                    CmdPrefixParser::new(
                                        IoRedirectParser::new(
                                            IoFileParser::new(WordParser::new()),
                                            IoHereParser::new(WordParser::new()),
                                        ),
                                        AssignmentWordParser::new(),
                                    ),
                                    CmdSuffixParser::new(
                                        WordParser::new(),
                                        IoRedirectParser::new(
                                            IoFileParser::new(WordParser::new()),
                                            IoHereParser::new(WordParser::new()),
                                        ),
                                    ),
                                )),
                                LinebreakParser::new(NewlineListParser::new()),
                            )),
                            LinebreakParser::new(NewlineListParser::new()),
                        ),
                        SeparatorOpParser::new(),
                    ),
                    SeparatorOpParser::new(),
                ),
                NewlineListParser::new(),
            ),
            LinebreakParser::new(NewlineListParser::new()),
        );

        Self {
            complete_command_parser,
            program_parser,
            lexer_adapter: LexerAdapter::new(lexer),
            options,
        }
    }

    /// Returns a parsed [`Program`].
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let maybe_program = self.program_parser.parse(&mut self.lexer_adapter);

        // Verify that no cached non-EOF tokens remain.
        // If such tokens are present, parsing is incomplete.
        for cached_token in self.lexer_adapter.clear_cache() {
            if cached_token != Token::EOF {
                return Err(ParseError::UnconsumedToken(cached_token));
            }
        }

        // Allow the parsed program to be verbosely written to stderr when requested.
        if self.options.borrow().debug_parsing {
            eprintln!("[pjsh::parser] {:?}", maybe_program);
        }

        maybe_program
    }

    /// Returns a parsed [`CompleteCommand`].
    pub fn parse_complete_command(&mut self) -> Result<CompleteCommand, ParseError> {
        let maybe_complete_command = self.complete_command_parser.parse(&mut self.lexer_adapter);

        // Verify that no cached non-EOF tokens remain.
        // If such tokens are present, parsing is incomplete.
        for token in self.lexer_adapter.clear_cache() {
            // TODO: Check that newline is not after EOF.
            if token != Token::Newline && token != Token::EOF {
                return Err(ParseError::UnconsumedToken(token));
            }
        }

        // Allow the parsed complete_command to be verbosely written to stderr when requested.
        if self.options.borrow().debug_parsing {
            eprintln!("[pjsh::parser] {:?}", maybe_complete_command);
        }

        maybe_complete_command
    }
}
