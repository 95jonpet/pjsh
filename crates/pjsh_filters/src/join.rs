use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter for joining lists into words using a separator.
#[derive(Debug, Clone)]
pub struct JoinFilter;
impl Filter for JoinFilter {
    fn name(&self) -> &str {
        "join"
    }

    fn filter_list(&self, list: Vec<String>, args: &[String]) -> FilterResult {
        match &args {
            [] => Err(FilterError::MissingArg("separator")),
            [separator] => Ok(Value::Word(list.join(separator))),
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
            JoinFilter.filter_list(vec!["item".into()], &[]),
            Err(FilterError::MissingArg("separator"))
        );
        assert_eq!(
            JoinFilter.filter_list(vec!["item".into()], &["1".into(), "2".into()]),
            Err(FilterError::TooManyArgs)
        );
    }

    #[test]
    fn it_joins_words() -> Result<(), FilterError> {
        let filter = JoinFilter;

        assert_eq!(
            filter.filter_list(vec!["single".into()], &[",".into()])?,
            Value::Word("single".into())
        );

        assert_eq!(
            filter.filter_list(vec!["first".into(), "second".into()], &[" ".into()])?,
            Value::Word("first second".into())
        );

        assert_eq!(
            filter.filter_list(
                vec!["first".into(), "".into(), "third".into()],
                &[",".into()]
            )?,
            Value::Word("first,,third".into())
        );

        Ok(())
    }
}
