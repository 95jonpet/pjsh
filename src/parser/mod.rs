mod error;

use std::collections::VecDeque;

use crate::{
    ast::{
        AndOr, AndOrPart, AssignmentWord, CmdPrefix, CmdSuffix, Command, CompleteCommand,
        CompleteCommands, IoFile, IoHere, IoRedirect, List, ListPart, PipeSequence, Pipeline,
        Program, RedirectList, SeparatorOp, SimpleCommand, Word, Wordlist,
    },
    lexer::{Lex, Mode},
    token::Token,
};

use self::error::ParseError;

pub struct Parser {
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

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        self.program()
    }

    fn assignment_word(&mut self) -> Result<AssignmentWord, ParseError> {
        match self.eat_word() {
            Ok(Word(word)) if word.contains('=') => {
                let split_index = word.find('=').unwrap();
                let key = String::from(&word[..split_index]);
                let value = String::from(&word[(split_index + 1)..]);
                Ok(AssignmentWord(key, value))
            }
            Ok(Word(non_assignment_word)) => {
                // Word token is not an assignment word.
                // The token has already been popped. Re-insert it in the cache manually.
                // This will miss any quotation tokens that were used to delimit the token.
                // TODO: Improve the robustness.
                self.cached_tokens
                    .push_front(Token::Word(non_assignment_word));
                Err(ParseError::UnexpectedCharSequence)
            }
            _ => Err(ParseError::UnexpectedToken(self.peek_token().clone())),
        }
    }

    // program          : linebreak complete_commands linebreak
    //                  | linebreak
    //                  ;
    fn program(&mut self) -> Result<Program, ParseError> {
        self.linebreak()?;
        if let Ok(complete_commands) = self.complete_commands() {
            self.linebreak()?;
            self.cached_tokens.clear();
            return Ok(Program(complete_commands));
        }
        self.cached_tokens.clear();
        Ok(Program(CompleteCommands(Vec::new())))
    }

    // complete_commands: complete_commands newline_list complete_command
    //                  |                                complete_command
    //                  ;
    fn complete_commands(&mut self) -> Result<CompleteCommands, ParseError> {
        let mut commands = Vec::new();
        while let Ok(command) = self.complete_command() {
            if self.linebreak().is_err() {
                return Err(ParseError::UnexpectedToken(self.peek_token().clone()));
            }

            commands.push(command);
        }

        Ok(CompleteCommands(commands))
    }

    // complete_command : list separator_op
    //                  | list
    //                  ;
    fn complete_command(&mut self) -> Result<CompleteCommand, ParseError> {
        let list = self.list()?;
        match self.separator_op() {
            Ok(separator_op) => Ok(CompleteCommand(list, Some(separator_op))),
            _ => Ok(CompleteCommand(list, None)),
        }
    }

    // list             : list separator_op and_or
    //                  |                   and_or
    //                  ;
    fn list(&mut self) -> Result<List, ParseError> {
        let mut parts = Vec::new();

        if let Ok(and_or) = self.and_or() {
            parts.push(ListPart::Start(and_or));
        }

        while let Ok(separator_op) = self.separator_op() {
            if let Ok(and_or) = self.and_or() {
                parts.push(ListPart::Tail(and_or, separator_op));
            } else {
                return Err(ParseError::UnexpectedToken(self.peek_token().clone()));
            }
        }

        if parts.is_empty() {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        } else {
            Ok(List(parts))
        }
    }

    // and_or           :                         pipeline
    //                  | and_or AND_IF linebreak pipeline
    //                  | and_or OR_IF  linebreak pipeline
    //                  ;
    fn and_or(&mut self) -> Result<AndOr, ParseError> {
        let mut parts = Vec::new();

        if let Ok(pipeline) = self.pipeline() {
            parts.push(AndOrPart::Start(pipeline));
        }

        loop {
            match self.peek_token() {
                Token::AndIf => {
                    self.next_token();
                    if let Ok(pipeline) = self.pipeline() {
                        parts.push(AndOrPart::And(pipeline));
                    } else {
                        self.cached_tokens.push_front(Token::AndIf);
                    }
                }
                Token::OrIf => {
                    self.next_token();
                    if let Ok(pipeline) = self.pipeline() {
                        parts.push(AndOrPart::Or(pipeline));
                    } else {
                        self.cached_tokens.push_front(Token::OrIf);
                    }
                }
                _ => break,
            }
        }

        if parts.is_empty() {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        } else {
            Ok(AndOr(parts))
        }
    }

    // pipeline         :      pipe_sequence
    //                  | Bang pipe_sequence
    //                  ;
    fn pipeline(&mut self) -> Result<Pipeline, ParseError> {
        if let Ok(Word(word)) = self.eat_word() {
            if &word == "!" {
                if let Ok(pipe_sequence) = self.pipe_sequence() {
                    Ok(Pipeline::Bang(pipe_sequence))
                } else {
                    Err(ParseError::UnexpectedToken(self.peek_token().clone()))
                }
            } else {
                // Unwanted token. Push it back to the front of the cache.
                // TODO: Make this more robust.
                self.cached_tokens.push_front(Token::Word(word));
                if let Ok(pipe_sequence) = self.pipe_sequence() {
                    Ok(Pipeline::Normal(pipe_sequence))
                } else {
                    Err(ParseError::UnexpectedToken(self.peek_token().clone()))
                }
            }
        } else {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        }
    }

    // pipe_sequence    :                             command
    //                  | pipe_sequence '|' linebreak command
    //                  ;
    fn pipe_sequence(&mut self) -> Result<PipeSequence, ParseError> {
        let mut commands = Vec::new();

        if let Ok(command) = self.command() {
            commands.push(command);
        }

        while self.peek_token() == &Token::Pipe {
            self.next_token();
            if self.linebreak().is_ok() {
                if let Ok(command) = self.command() {
                    commands.push(command);
                }
            }
        }

        if commands.is_empty() {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        } else {
            Ok(PipeSequence(commands))
        }
    }

    // command          : simple_command
    //                  | compound_command
    //                  | compound_command redirect_list
    //                  | function_definition
    //                  ;
    fn command(&mut self) -> Result<Command, ParseError> {
        if let Ok(simple_command) = self.simple_command() {
            Ok(Command::Simple(simple_command))
        } else {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        }

        // TODO: Add support for all variants.
    }

    // name             : NAME                     /* Apply rule 5 */
    //                  ;
    fn name(&mut self) -> Result<Word, ParseError> {
        self.eat_word()
    }

    // wordlist         : wordlist WORD
    //                  |          WORD
    //                  ;
    fn wordlist(&mut self) -> Result<Wordlist, ParseError> {
        let mut words = Vec::new();
        loop {
            match self.eat_word() {
                Ok(word) => words.push(word),
                _ => break,
            }
        }

        if words.is_empty() {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        } else {
            Ok(Wordlist(words))
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

    // simple_command   : cmd_prefix cmd_word cmd_suffix
    //                  | cmd_prefix cmd_word
    //                  | cmd_prefix
    //                  | cmd_name cmd_suffix
    //                  | cmd_name
    //                  ;
    fn simple_command(&mut self) -> Result<SimpleCommand, ParseError> {
        if let Ok(prefix) = self.cmd_prefix() {
            if let Ok(Word(cmd_word)) = self.cmd_word() {
                if let Ok(suffix) = self.cmd_suffix() {
                    Ok(SimpleCommand(Some(prefix), Some(cmd_word), Some(suffix)))
                } else {
                    Ok(SimpleCommand(Some(prefix), Some(cmd_word), None))
                }
            } else {
                Ok(SimpleCommand(Some(prefix), None, None))
            }
        } else if let Ok(Word(cmd_name)) = self.cmd_name() {
            if let Ok(suffix) = self.cmd_suffix() {
                Ok(SimpleCommand(None, Some(cmd_name), Some(suffix)))
            } else {
                Ok(SimpleCommand(None, Some(cmd_name), None))
            }
        } else {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        }
    }

    // cmd_prefix       :            io_redirect
    //                  | cmd_prefix io_redirect
    //                  |            ASSIGNMENT_WORD
    //                  | cmd_prefix ASSIGNMENT_WORD
    //                  ;
    fn cmd_prefix(&mut self) -> Result<CmdPrefix, ParseError> {
        let mut assignments = Vec::new();
        let mut redirects = Vec::new();

        loop {
            if let Ok(redirect) = self.io_redirect() {
                redirects.push(redirect);
                continue;
            }

            if let Ok(assignment) = self.assignment_word() {
                assignments.push(assignment);
                continue;
            }

            break;
        }

        if assignments.is_empty() && redirects.is_empty() {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        } else {
            Ok(CmdPrefix(assignments, RedirectList(redirects)))
        }
    }

    // cmd_suffix       :            io_redirect
    //                  | cmd_suffix io_redirect
    //                  |            WORD
    //                  | cmd_suffix WORD
    //                  ;
    fn cmd_suffix(&mut self) -> Result<CmdSuffix, ParseError> {
        let mut redirects = Vec::new();
        let mut words = Vec::new();

        loop {
            if let Ok(redirect) = self.io_redirect() {
                redirects.push(redirect);
                continue;
            }

            if let Ok(word) = self.eat_word() {
                words.push(word);
                continue;
            }

            break;
        }

        if redirects.is_empty() && words.is_empty() {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        } else {
            Ok(CmdSuffix(Wordlist(words), RedirectList(redirects)))
        }
    }

    // redirect_list    :               io_redirect
    //                  | redirect_list io_redirect
    //                  ;
    fn redirect_list(&mut self) -> Result<RedirectList, ParseError> {
        let mut redirects = Vec::new();
        loop {
            match self.io_redirect() {
                Ok(redirect) => redirects.push(redirect),
                _ => break,
            }
        }

        if redirects.is_empty() {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
        } else {
            Ok(RedirectList(redirects))
        }
    }

    // io_redirect      :           io_file
    //                  | IO_NUMBER io_file
    //                  |           io_here
    //                  | IO_NUMBER io_here
    //                  ;
    fn io_redirect(&mut self) -> Result<IoRedirect, ParseError> {
        let mut io_number: Option<u8> = None;
        match self.peek_token() {
            Token::IoNumber(_) => {
                if let Token::IoNumber(number) = self.next_token() {
                    io_number = Some(number);
                }
            }
            _ => (),
        }

        if let Ok(io_file) = self.io_file() {
            Ok(IoRedirect::IoFile(io_number, io_file))
        } else if let Ok(io_here) = self.io_here() {
            Ok(IoRedirect::IoHere(io_number, io_here))
        } else {
            Err(ParseError::UnexpectedToken(self.peek_token().clone()))
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
            _ => Err(ParseError::UnexpectedToken(self.peek_token().clone())),
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
    fn io_here(&mut self) -> Result<IoHere, ParseError> {
        match self.peek_token() {
            Token::DLess => {
                self.next_token();
                self.here_end().map(|Word(end)| IoHere::DLess(end))
            }
            Token::DLessDash => {
                self.next_token();
                self.here_end().map(|Word(end)| IoHere::DLessDash(end))
            }
            _ => Err(ParseError::UnexpectedToken(self.peek_token().clone())),
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
        self.newline_list().or(Ok(()))
    }

    // separator_op     : '&'
    //                  | ';'
    //                  ;
    fn separator_op(&mut self) -> Result<SeparatorOp, ParseError> {
        match self.peek_token() {
            Token::And => {
                self.next_token();
                Ok(SeparatorOp::Async)
            }
            Token::Semi => {
                self.next_token();
                Ok(SeparatorOp::Serial)
            }
            _ => Err(ParseError::UnexpectedToken(self.peek_token().clone())),
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
                        _ => Err(ParseError::UnexpectedToken(self.peek_token().clone())),
                    },
                    _ => Err(ParseError::UnexpectedToken(self.peek_token().clone())),
                }
            }
            _ => Err(ParseError::UnexpectedToken(self.peek_token().clone())),
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
            parser(tokens).wordlist()
        );
    }

    #[test]
    fn it_parses_simple_command() {
        let tokens = vec![
            Token::Word(String::from("key=value")),
            Token::Word(String::from("command")),
            Token::Word(String::from("argument")),
            Token::IoNumber(2),
            Token::Great,
            Token::Word(String::from("error_file")),
        ];

        assert_eq!(
            Ok(SimpleCommand(
                Some(CmdPrefix(
                    vec![AssignmentWord(String::from("key"), String::from("value"))],
                    RedirectList(Vec::new())
                )),
                Some(String::from("command")),
                Some(CmdSuffix(
                    Wordlist(vec![Word(String::from("argument"))]),
                    RedirectList(vec![IoRedirect::IoFile(
                        Some(2),
                        IoFile::Great(String::from("error_file"))
                    )])
                ))
            )),
            parser(tokens).simple_command()
        );
    }

    #[test]
    fn it_parses_cmd_prefix() {
        let tokens = vec![
            Token::Word(String::from("key=value")),
            Token::IoNumber(2),
            Token::Great,
            Token::Word(String::from("error_file")),
        ];

        assert_eq!(
            Ok(CmdPrefix(
                vec![AssignmentWord(String::from("key"), String::from("value")),],
                RedirectList(vec![IoRedirect::IoFile(
                    Some(2),
                    IoFile::Great(String::from("error_file"))
                ),],)
            )),
            parser(tokens).cmd_prefix()
        );
    }

    #[test]
    fn it_parses_cmd_suffix() {
        let tokens = vec![
            Token::Word(String::from("argument1")),
            Token::IoNumber(2),
            Token::Great,
            Token::Word(String::from("error_file")),
            Token::Word(String::from("argument2")),
            Token::Less,
            Token::Word(String::from("in_file")),
        ];

        assert_eq!(
            Ok(CmdSuffix(
                Wordlist(vec![
                    Word(String::from("argument1")),
                    Word(String::from("argument2"))
                ]),
                RedirectList(vec![
                    IoRedirect::IoFile(Some(2), IoFile::Great(String::from("error_file"))),
                    IoRedirect::IoFile(None, IoFile::Less(String::from("in_file")))
                ],)
            )),
            parser(tokens).cmd_suffix()
        );
    }

    #[test]
    fn it_parses_redirect_list() {
        let tokens = vec![
            Token::IoNumber(1),
            Token::Great,
            Token::Word(String::from("file1")),
            Token::IoNumber(2),
            Token::DGreat,
            Token::Word(String::from("file2")),
        ];
        assert_eq!(
            Ok(RedirectList(vec![
                IoRedirect::IoFile(Some(1), IoFile::Great(String::from("file1"))),
                IoRedirect::IoFile(Some(2), IoFile::DGreat(String::from("file2")))
            ])),
            parser(tokens).redirect_list()
        );
    }

    #[test]
    fn it_parses_io_redirect() {
        let tokens = vec![
            Token::IoNumber(1),
            Token::Great,
            Token::Word(String::from("file")),
        ];
        assert_eq!(
            Ok(IoRedirect::IoFile(
                Some(1),
                IoFile::Great(String::from("file"))
            )),
            parser(tokens).io_redirect()
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
    fn it_parses_io_here() {
        let prefix_tokens = [
            (Token::DLess, IoHere::DLess(String::from("end"))),
            (Token::DLessDash, IoHere::DLessDash(String::from("end"))),
        ];

        for (prefix, io_here) in prefix_tokens {
            assert_eq!(
                Ok(io_here),
                parser(vec![prefix, Token::Word(String::from("end"))]).io_here()
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
        assert_eq!(
            Ok(SeparatorOp::Serial),
            parser(vec![Token::Semi]).separator_op()
        );
        assert_eq!(
            Ok(SeparatorOp::Async),
            parser(vec![Token::And]).separator_op()
        );
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
