use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter that reverses lists.
#[derive(Debug, Clone)]
pub struct ReverseFilter;
impl Filter for ReverseFilter {
    fn name(&self) -> &str {
        "reverse"
    }

    fn filter_list(&self, mut list: Vec<String>, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        list.reverse();
        Ok(Value::List(list))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_no_args() {
        assert_eq!(
            ReverseFilter.filter_list(vec!["item".into()], &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
    }

    #[test]
    fn it_reverses_lists() -> Result<(), FilterError> {
        let filter = ReverseFilter;

        assert_eq!(
            filter.filter_list(vec!["a".into(), "b".into(), "c".into()], &[])?,
            Value::List(vec!["c".into(), "b".into(), "a".into()])
        );

        Ok(())
    }
}
