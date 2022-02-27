use lazy_static::lazy_static;

use std::collections::HashMap;

use crate::token::TokenContents;

lazy_static! {
    /// Map of operator tokens that can be parsed from a well-defined string.
    static ref OPERATORS: HashMap<&'static str, TokenContents<'static>> = {
        let mut ops = HashMap::new();

        ops.insert("(", TokenContents::OpenParen);
        ops.insert(")", TokenContents::CloseParen);
        ops.insert("{", TokenContents::OpenBrace);
        ops.insert("}", TokenContents::CloseBrace);
        ops.insert("[[", TokenContents::DoubleOpenBracket);
        ops.insert("]]", TokenContents::DoubleCloseBracket);

        ops.insert("&&", TokenContents::AndIf);
        ops.insert("||", TokenContents::OrIf);

        ops.insert("&", TokenContents::Amp);
        ops.insert("|", TokenContents::Pipe);
        ops.insert("->|", TokenContents::PipeStart);
        ops.insert(";", TokenContents::Semi);

        ops.insert("\"", TokenContents::Quote);
        ops.insert("'", TokenContents::Quote);
        ops.insert("\"\"\"", TokenContents::TripleQuote);
        ops.insert("'''", TokenContents::TripleQuote);

        ops
    };
}

/// Returns all potential operators that match some input along with the input corresponing to the
/// operator. The potential operators are ordered by length from shortest to longest.
pub fn potential_operators(input: &str) -> Vec<(&'static str, TokenContents<'static>)> {
    let mut ops: Vec<(&'static str, TokenContents<'static>)> = OPERATORS
        .iter()
        .filter(|(k, _)| k.starts_with(input))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    ops.sort_unstable_by_key(|(k, _)| k.len());

    ops
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenContents::*;
    use potential_operators as ops;

    #[test]
    fn it_returns_potential_operators() {
        assert_eq!(ops("|"), vec![("|", Pipe), ("||", OrIf)]);

        // Test some quote operators.
        assert_eq!(ops("'"), vec![("'", Quote), ("'''", TripleQuote)]);
        assert_eq!(ops("''"), vec![("'''", TripleQuote)]);
        assert_eq!(ops("'''"), vec![("'''", TripleQuote)]);
        assert_eq!(ops("''''"), vec![]);

        // Edge cases.
        assert_eq!(ops("a|"), vec![]);
        assert_eq!(ops("(|"), vec![]);
    }
}
