use lazy_static::lazy_static;
use pjsh_ast::{Iterable, NumericRange};
use regex::Regex;

use crate::ParseError;

/// Parses an iterable.
pub(crate) fn parse_iterable(word: &str) -> Result<Iterable, ParseError> {
    if let Some(numeric_range) = parse_numeric_range(word) {
        return Ok(Iterable::NumericRange(numeric_range));
    }

    Err(ParseError::InvalidSyntax(format!(
        "Could not parse iterable: {word}"
    )))
}

/// Parses a numeric range iterable.
fn parse_numeric_range(word: &str) -> Option<NumericRange> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(-?\d+)\.\.(=?)(-?\d+)"#).expect("Compile regex");
    }

    if let Some(captures) = RE.captures(word) {
        let start = captures[1].parse::<isize>();
        let is_end_included = &captures[2] == "=";
        let end = captures[3].parse::<isize>();

        if let (Ok(start), Ok(end)) = (start, end) {
            let end = match is_end_included {
                true if start > end => end - 1, // Decrementing one more.
                true => end + 1,                // Incrementing one more.
                false => end,
            };

            return Some(NumericRange::new(start, end));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_range() {
        assert!(parse_iterable("..").is_err());
    }

    #[test]
    fn parse_numeric_range() -> Result<(), ParseError> {
        let range = |start, end| Iterable::NumericRange(NumericRange::new(start, end));

        // Increasing order.
        assert_eq!(parse_iterable("0..0"), Ok(range(0, 0)));
        assert_eq!(parse_iterable("0..=0"), Ok(range(0, 1)));
        assert_eq!(parse_iterable("0..1"), Ok(range(0, 1)));
        assert_eq!(parse_iterable("0..3"), Ok(range(0, 3)));
        assert_eq!(parse_iterable("-1..-1"), Ok(range(-1, -1)));
        assert_eq!(parse_iterable("-1..0"), Ok(range(-1, 0)));
        assert_eq!(parse_iterable("-1..=1"), Ok(range(-1, 2)));

        // Decreasing order.
        assert_eq!(parse_iterable("1..0"), Ok(range(1, 0)));
        assert_eq!(parse_iterable("1..=0"), Ok(range(1, -1)));
        assert_eq!(parse_iterable("3..0"), Ok(range(3, 0)));
        assert_eq!(parse_iterable("0..-1"), Ok(range(0, -1)));
        assert_eq!(parse_iterable("1..=-1"), Ok(range(1, -2)));

        Ok(())
    }
}