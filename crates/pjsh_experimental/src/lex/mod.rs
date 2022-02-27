mod operator;

use crate::{error::LexError, lex::operator::potential_operators, token::Token};

type Chars<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

pub fn lex<'a>(input: &'a str) -> Result<Vec<Token<'a>>, LexError> {
    let mut chars = input.char_indices().peekable();
    let mut tokens = Vec::new();

    while chars.peek().is_some() {
        tokens.push(next_token(&mut chars, input)?);
    }

    Ok(tokens)
}

fn next_token<'a>(chars: &mut Chars<'a>, input: &'a str) -> Result<Token<'a>, LexError> {
    let mut op_chars = chars.clone();
    let op_start = op_chars.peek().unwrap().0;
    let mut op = &input[op_start..op_chars.peek().unwrap().0];
    let mut tokens = potential_operators(op);
    todo!()
}
