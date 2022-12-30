use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter for separating words into lists based on lines.
///
/// Empty words are removed.
#[derive(Debug, Clone)]
pub struct WordsFilter;
impl Filter for WordsFilter {
    fn name(&self) -> &str {
        "words"
    }

    fn filter_word(&self, word: String, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        let words = word
            .split(char::is_whitespace)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect();

        Ok(Value::List(words))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_no_args() {
        assert_eq!(
            WordsFilter.filter_word("word".into(), &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
    }

    #[test]
    fn it_returns_words() -> Result<(), FilterError> {
        let filter = WordsFilter;

        assert_eq!(filter.filter_word("".into(), &[])?, Value::List(vec![]));

        assert_eq!(
            filter.filter_word("a b\tc\nd".into(), &[])?,
            Value::List(vec!["a".into(), "b".into(), "c".into(), "d".into()])
        );

        Ok(())
    }
}
