use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter that returns the first word in a list.
#[derive(Debug, Clone)]
pub struct FirstFilter;
impl Filter for FirstFilter {
    fn name(&self) -> &str {
        "first"
    }

    fn filter_list(&self, list: Vec<String>, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        let Some(item) = list.into_iter().next() else {
            return Err(FilterError::NoSuchValue);
        };

        Ok(Value::Word(item))
    }
}

/// A filter that returns the last word in a list.
#[derive(Debug, Clone)]
pub struct LastFilter;
impl Filter for LastFilter {
    fn name(&self) -> &str {
        "last"
    }

    fn filter_list(&self, list: Vec<String>, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        let Some(item) = list.into_iter().last() else {
            return Err(FilterError::NoSuchValue);
        };

        Ok(Value::Word(item))
    }
}

/// A filter that returns the `n`-th word in a list.
#[derive(Debug, Clone)]
pub struct NthFilter;
impl Filter for NthFilter {
    fn name(&self) -> &str {
        "nth"
    }

    fn filter_list(&self, list: Vec<String>, args: &[String]) -> FilterResult {
        let n = match &args {
            [] => return Err(FilterError::MissingArg("index")),
            [n] => match n.parse::<usize>() {
                Ok(n) => n,
                Err(err) => return Err(FilterError::InvalidArgs(format!("invalid index: {err}"))),
            },
            _ => return Err(FilterError::TooManyArgs),
        };

        let Some(item) = list.into_iter().nth(n) else {
            return Err(FilterError::NoSuchValue);
        };

        Ok(Value::Word(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_args() {
        assert_eq!(
            NthFilter.filter_list(vec!["item".into()], &[]),
            Err(FilterError::MissingArg("index"))
        );
        assert_eq!(
            NthFilter.filter_list(vec!["item".into()], &["1".into(), "2".into()]),
            Err(FilterError::TooManyArgs)
        );

        assert_eq!(
            FirstFilter.filter_list(vec!["item".into()], &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );

        assert_eq!(
            LastFilter.filter_list(vec!["item".into()], &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
    }

    #[test]
    fn it_returns_the_nth_item() -> Result<(), FilterError> {
        assert_eq!(
            NthFilter.filter_list(vec!["single".into()], &["0".into()])?,
            Value::Word("single".into())
        );
        assert_eq!(
            NthFilter.filter_list(vec!["single".into()], &["1".into()]),
            Err(FilterError::NoSuchValue)
        );

        assert_eq!(
            NthFilter.filter_list(vec!["first".into(), "second".into()], &["1".into()])?,
            Value::Word("second".into())
        );

        assert_eq!(
            NthFilter.filter_list(
                vec!["first".into(), "".into(), "third".into()],
                &["2".into()]
            )?,
            Value::Word("third".into())
        );

        assert!(matches!(
            NthFilter.filter_list(vec!["first".into(), "second".into()], &["n".into()]),
            Err(FilterError::InvalidArgs(_))
        ));

        Ok(())
    }

    #[test]
    fn it_returns_the_first_item() -> Result<(), FilterError> {
        assert_eq!(
            FirstFilter.filter_list(vec![], &[]),
            Err(FilterError::NoSuchValue)
        );

        assert_eq!(
            FirstFilter.filter_list(vec!["single".into()], &[])?,
            Value::Word("single".into())
        );

        assert_eq!(
            FirstFilter.filter_list(vec!["first".into(), "second".into()], &[])?,
            Value::Word("first".into())
        );

        assert_eq!(
            FirstFilter.filter_list(vec!["first".into(), "second".into(), "third".into()], &[])?,
            Value::Word("first".into())
        );

        Ok(())
    }

    #[test]
    fn it_returns_the_last_item() -> Result<(), FilterError> {
        assert_eq!(
            LastFilter.filter_list(vec![], &[]),
            Err(FilterError::NoSuchValue)
        );

        assert_eq!(
            LastFilter.filter_list(vec!["single".into()], &[])?,
            Value::Word("single".into())
        );

        assert_eq!(
            LastFilter.filter_list(vec!["first".into(), "second".into()], &[])?,
            Value::Word("second".into())
        );

        assert_eq!(
            LastFilter.filter_list(vec!["first".into(), "second".into(), "third".into()], &[])?,
            Value::Word("third".into())
        );

        Ok(())
    }
}
