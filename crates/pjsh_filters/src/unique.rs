use itertools::Itertools;
use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter that removes duplicate values from lists.
#[derive(Debug, Clone)]
pub struct UniqueFilter;
impl Filter for UniqueFilter {
    fn name(&self) -> &str {
        "unique"
    }

    fn filter_list(&self, list: Vec<String>, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        Ok(Value::List(list.into_iter().unique().collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_no_args() {
        assert_eq!(
            UniqueFilter.filter_list(vec!["item".into()], &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
    }

    #[test]
    fn it_removes_duplicated_list_items() -> Result<(), FilterError> {
        let filter = UniqueFilter;

        assert_eq!(
            filter.filter_list(vec!["a".into(), "b".into(), "a".into()], &[])?,
            Value::List(vec!["a".into(), "b".into()])
        );

        Ok(())
    }
}
