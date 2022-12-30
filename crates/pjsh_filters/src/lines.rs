use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter for separating words into lists based on lines.
///
/// Lines are ended with either a newline (`\n`) or a carriage return with
/// a line feed (`\r\n`).
#[derive(Debug, Clone)]
pub struct LinesFilter;
impl Filter for LinesFilter {
    fn name(&self) -> &str {
        "lines"
    }

    fn filter_word(&self, word: String, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        Ok(Value::List(word.lines().map(ToString::to_string).collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_no_args() {
        let filter = LinesFilter;
        assert_eq!(
            filter.filter_word("word".into(), &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
    }

    #[test]
    fn it_returns_lines() -> Result<(), FilterError> {
        let filter = LinesFilter;

        assert_eq!(filter.filter_word("".into(), &[])?, Value::List(vec![]));

        assert_eq!(
            filter.filter_word("a\nb\r\nc".into(), &[])?,
            Value::List(vec!["a".into(), "b".into(), "c".into()])
        );

        Ok(())
    }
}
