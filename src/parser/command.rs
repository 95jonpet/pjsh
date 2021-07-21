use crate::ast::{
    CmdPrefix, CmdSuffix, Command, CompleteCommand, CompleteCommands, Program, RedirectList,
    SimpleCommand, Word, Wordlist,
};

use super::{
    adapter::LexerAdapter,
    error::ParseError,
    io::IoRedirectParser,
    meta::{LinebreakParser, NewlineListParser, SeparatorOpParser},
    pipeline::ListParser,
    word::{AssignmentWordParser, WordParser},
    Parse,
};

/// Parses [`crate::ast::Command`] syntax.
///```yacc
/// command          : simple_command
///                  | compound_command
///                  | compound_command redirect_list
///                  | function_definition
///                  ;
///```
pub(crate) struct CommandParser {
    simple_command_parser: SimpleCommandParser,
}

impl CommandParser {
    pub fn new(simple_command_parser: SimpleCommandParser) -> Self {
        Self {
            simple_command_parser,
        }
    }
}

impl Parse for CommandParser {
    type Item = Command;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        // TODO: Add support for all variants.
        if let Ok(simple_command) = self.simple_command_parser.parse(lexer) {
            Ok(Command::Simple(simple_command))
        } else {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        }
    }
}

/// Parses [`crate::ast::CompleteCommand`] syntax.
///```yacc
/// complete_command : list separator_op
///                  | list
///                  ;
///```
pub(crate) struct CompleteCommandParser {
    list_parser: ListParser,
    separator_op_parser: SeparatorOpParser,
}

impl CompleteCommandParser {
    pub fn new(list_parser: ListParser, separator_op_parser: SeparatorOpParser) -> Self {
        Self {
            list_parser,
            separator_op_parser,
        }
    }
}

impl Parse for CompleteCommandParser {
    type Item = CompleteCommand;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let list = self.list_parser.parse(lexer)?;
        match self.separator_op_parser.parse(lexer) {
            Ok(separator_op) => Ok(CompleteCommand(list, Some(separator_op))),
            _ => Ok(CompleteCommand(list, None)),
        }
    }
}

/// Parses [`crate::ast::CompleteCommands`] syntax.
///```yacc
/// complete_commands: complete_commands newline_list complete_command
///                  |                                complete_command
///                  ;
///```
pub(crate) struct CompleteCommandsParser {
    complete_command_parser: CompleteCommandParser,
    newline_list_parser: NewlineListParser,
}

impl CompleteCommandsParser {
    pub fn new(
        complete_command_parser: CompleteCommandParser,
        newline_list_parser: NewlineListParser,
    ) -> Self {
        Self {
            complete_command_parser,
            newline_list_parser,
        }
    }
}

impl Parse for CompleteCommandsParser {
    type Item = CompleteCommands;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut commands = Vec::new();
        while let Ok(command) = self.complete_command_parser.parse(lexer) {
            if self.newline_list_parser.parse(lexer).is_err() {
                return Err(ParseError::UnexpectedToken(lexer.peek_token().clone()));
            }

            commands.push(command);
        }

        Ok(CompleteCommands(commands))
    }
}

/// Parses [`crate::ast::CmdPrefix`] syntax.
///```yacc
/// cmd_prefix       :            io_redirect
///                  | cmd_prefix io_redirect
///                  |            ASSIGNMENT_WORD
///                  | cmd_prefix ASSIGNMENT_WORD
///                  ;
///```
pub(crate) struct CmdPrefixParser {
    io_redirect_parser: IoRedirectParser,
    assignment_word_parser: AssignmentWordParser,
}

impl CmdPrefixParser {
    pub fn new(
        io_redirect_parser: IoRedirectParser,
        assignment_word_parser: AssignmentWordParser,
    ) -> Self {
        Self {
            io_redirect_parser,
            assignment_word_parser,
        }
    }
}

impl Parse for CmdPrefixParser {
    type Item = CmdPrefix;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut assignments = Vec::new();
        let mut redirects = Vec::new();

        loop {
            if let Ok(redirect) = self.io_redirect_parser.parse(lexer) {
                redirects.push(redirect);
                continue;
            }

            if let Ok(assignment) = self.assignment_word_parser.parse(lexer) {
                assignments.push(assignment);
                continue;
            }

            break;
        }

        if assignments.is_empty() && redirects.is_empty() {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        } else {
            Ok(CmdPrefix(assignments, RedirectList(redirects)))
        }
    }
}

/// Parses [`crate::ast::CmdSuffix`] syntax.
///```yacc
/// cmd_suffix       :            io_redirect
///                  | cmd_suffix io_redirect
///                  |            WORD
///                  | cmd_suffix WORD
///                  ;
///```
pub(crate) struct CmdSuffixParser {
    word_parser: WordParser,
    io_redirect_parser: IoRedirectParser,
}

impl CmdSuffixParser {
    pub fn new(word_parser: WordParser, io_redirect_parser: IoRedirectParser) -> Self {
        Self {
            word_parser,
            io_redirect_parser,
        }
    }
}

impl Parse for CmdSuffixParser {
    type Item = CmdSuffix;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        let mut redirects = Vec::new();
        let mut words = Vec::new();

        loop {
            if let Ok(redirect) = self.io_redirect_parser.parse(lexer) {
                redirects.push(redirect);
                continue;
            }

            if let Ok(word) = self.word_parser.parse(lexer) {
                words.push(word);
                continue;
            }

            break;
        }

        if redirects.is_empty() && words.is_empty() {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        } else {
            Ok(CmdSuffix(Wordlist(words), RedirectList(redirects)))
        }
    }
}

/// Parses [`crate::ast::SimpleCommand`] syntax.
///```yacc
/// simple_command   : cmd_prefix cmd_word cmd_suffix
///                  | cmd_prefix cmd_word
///                  | cmd_prefix
///                  | cmd_name cmd_suffix
///                  | cmd_name
///                  ;
///```
pub(crate) struct SimpleCommandParser {
    word_parser: WordParser,
    cmd_prefix_parser: CmdPrefixParser,
    cmd_suffix_parser: CmdSuffixParser,
}

impl SimpleCommandParser {
    pub fn new(
        word_parser: WordParser,
        cmd_prefix_parser: CmdPrefixParser,
        cmd_suffix_parser: CmdSuffixParser,
    ) -> Self {
        Self {
            word_parser,
            cmd_prefix_parser,
            cmd_suffix_parser,
        }
    }

    fn cmd_name(&mut self, lexer: &mut LexerAdapter) -> Result<Word, ParseError> {
        self.word_parser.parse(lexer)
    }

    fn cmd_word(&mut self, lexer: &mut LexerAdapter) -> Result<Word, ParseError> {
        self.word_parser.parse(lexer)
    }
}

impl Parse for SimpleCommandParser {
    type Item = SimpleCommand;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        if let Ok(prefix) = self.cmd_prefix_parser.parse(lexer) {
            if let Ok(cmd_word) = self.cmd_word(lexer) {
                if let Ok(suffix) = self.cmd_suffix_parser.parse(lexer) {
                    Ok(SimpleCommand(Some(prefix), Some(cmd_word), Some(suffix)))
                } else {
                    Ok(SimpleCommand(Some(prefix), Some(cmd_word), None))
                }
            } else {
                Ok(SimpleCommand(Some(prefix), None, None))
            }
        } else if let Ok(cmd_name) = self.cmd_name(lexer) {
            if let Ok(suffix) = self.cmd_suffix_parser.parse(lexer) {
                Ok(SimpleCommand(None, Some(cmd_name), Some(suffix)))
            } else {
                Ok(SimpleCommand(None, Some(cmd_name), None))
            }
        } else {
            Err(ParseError::UnexpectedToken(lexer.peek_token().clone()))
        }
    }
}

/// Parses [`crate::ast::Program`] syntax.
///```yacc
/// program          : linebreak complete_commands linebreak
///                  | linebreak
///                  ;
///```
pub(crate) struct ProgramParser {
    complete_commands_parser: CompleteCommandsParser,
    linebreak_parser: LinebreakParser,
}

impl ProgramParser {
    pub fn new(
        complete_commands_parser: CompleteCommandsParser,
        linebreak_parser: LinebreakParser,
    ) -> Self {
        Self {
            complete_commands_parser,
            linebreak_parser,
        }
    }
}

impl Parse for ProgramParser {
    type Item = Program;

    fn parse(&mut self, lexer: &mut LexerAdapter) -> Result<Self::Item, ParseError> {
        self.linebreak_parser.parse(lexer)?;
        if let Ok(complete_commands) = self.complete_commands_parser.parse(lexer) {
            self.linebreak_parser.parse(lexer)?;
            Ok(Program(complete_commands))
        } else {
            Ok(Program(CompleteCommands(Vec::new())))
        }
    }
}
