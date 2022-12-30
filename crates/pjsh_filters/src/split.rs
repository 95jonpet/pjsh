use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter that splits words into lists using a separator.
#[derive(Debug, Clone)]
pub struct SplitFilter;
impl Filter for SplitFilter {
    fn name(&self) -> &str {
        "split"
    }

    fn filter_word(&self, word: String, args: &[String]) -> FilterResult {
        match &args {
            [] => Err(FilterError::MissingArg("separator")),
            [separator] => Ok(Value::List(
                word.split(separator)
                    .into_iter()
                    .map(ToString::to_string)
                    .collect(),
            )),
            _ => Err(FilterError::TooManyArgs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_one_arg() {
        assert_eq!(
            SplitFilter.filter_word("word".into(), &[]),
            Err(FilterError::MissingArg("separator"))
        );
        assert_eq!(
            SplitFilter.filter_word("word".into(), &["1".into(), "2".into()]),
            Err(FilterError::TooManyArgs)
        );
    }

    #[test]
    fn it_splits_words() -> Result<(), FilterError> {
        let filter = SplitFilter;

        assert_eq!(
            filter.filter_word("single".into(), &["sep".into()])?,
            Value::List(vec!["single".into()])
        );

        assert_eq!(
            filter.filter_word("first,second".into(), &[",".into()])?,
            Value::List(vec!["first".into(), "second".into()])
        );

        assert_eq!(
            filter.filter_word("first,,third".into(), &[",".into()])?,
            Value::List(vec!["first".into(), "".into(), "third".into()])
        );

        Ok(())
    }
}
