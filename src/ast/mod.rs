#[derive(Debug, PartialEq)]
pub struct Word(pub String);

#[derive(Debug, PartialEq)]
pub struct AssignmentWord(pub String, pub String);

#[derive(Debug, PartialEq)]
pub enum Pipeline {
    Normal(PipeSequence),
    Bang(PipeSequence)
}

#[derive(Debug, PartialEq)]
pub struct PipeSequence(pub Vec<Command>);

#[derive(Debug, PartialEq)]
pub enum Command {
    Simple(SimpleCommand),
}

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
