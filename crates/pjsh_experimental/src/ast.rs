#[derive(Debug, PartialEq)]
pub enum Word<'a> {
    Literal(&'a str),
    Quoted(&'a str),
    Variable(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct Command<'a>(pub Vec<Word<'a>>);
