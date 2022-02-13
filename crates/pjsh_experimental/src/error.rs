pub enum LexError {}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    IncompleteSequence,
    UnexpectedToken,
}
