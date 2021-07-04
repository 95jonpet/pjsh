/// Represents a regular word/string of characters.
/// Note that whitespace characters may be present.
#[derive(Debug, PartialEq)]
pub struct Word(pub String);

/// Represents an assignment.
#[derive(Debug, PartialEq)]
pub struct AssignmentWord(pub String, pub String);

#[derive(Debug, PartialEq)]
pub struct List(pub Vec<ListPart>);

#[derive(Debug, PartialEq)]
pub enum ListPart {
    Start(AndOr),
    Tail(AndOr, SeparatorOp),
}

#[derive(Debug, PartialEq)]
pub struct AndOr(pub Vec<AndOrPart>);

#[derive(Debug, PartialEq)]
pub enum AndOrPart {
    Start(Pipeline),
    And(Pipeline),
    Or(Pipeline),
}

#[derive(Debug, PartialEq)]
pub enum Pipeline {
    Normal(PipeSequence),
    Bang(PipeSequence),
}

/// Represents a sequence of commands separated by pipes.
#[derive(Debug, PartialEq)]
pub struct PipeSequence(pub Vec<Command>);

#[derive(Debug, PartialEq)]
pub struct Program(pub CompleteCommands);

#[derive(Debug, PartialEq)]
pub struct CompleteCommands(pub Vec<CompleteCommand>);

#[derive(Debug, PartialEq)]
pub struct CompleteCommand(pub List, pub Option<SeparatorOp>);

#[derive(Debug, PartialEq)]
pub enum Command {
    Simple(SimpleCommand),
}

/// Represents a command with an optional prefix and suffix.
#[derive(Debug, PartialEq)]
pub struct SimpleCommand(
    pub Option<CmdPrefix>,
    pub Option<String>,
    pub Option<CmdSuffix>,
);

#[derive(Debug, PartialEq)]
pub struct CmdPrefix(pub Vec<AssignmentWord>, pub RedirectList);

#[derive(Debug, PartialEq)]
pub struct CmdSuffix(pub Wordlist, pub RedirectList);

#[derive(Debug, PartialEq)]
pub struct Wordlist(pub Vec<Word>);

#[derive(Debug, PartialEq)]
pub struct RedirectList(pub Vec<IoRedirect>);

#[derive(Debug, PartialEq)]
pub enum IoFile {
    Less(String),
    LessAnd(String),
    Great(String),
    GreatAnd(String),
    DGreat(String),
    LessGreat(String),
    Clobber(String),
}

#[derive(Debug, PartialEq)]
pub enum IoHere {
    DLess(String),
    DLessDash(String),
}

#[derive(Debug, PartialEq)]
pub enum IoRedirect {
    IoFile(Option<u8>, IoFile),
    IoHere(Option<u8>, IoHere),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SeparatorOp {
    /// &
    Async,
    /// ;
    Serial,
}
