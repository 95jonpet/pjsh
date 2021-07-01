#[derive(Debug, PartialEq)]
pub struct Word(pub String);

#[derive(Debug, PartialEq)]
pub struct Wordlist(pub Vec<Word>);

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
    IoFile(u8, IoFile),
    IoHere(u8, IoHere),
}
