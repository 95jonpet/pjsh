use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter that sorts lists.
#[derive(Debug, Clone)]
pub struct SortFilter;
impl Filter for SortFilter {
    fn name(&self) -> &str {
        "sort"
    }

    fn filter_list(&self, mut list: Vec<String>, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        list.sort_unstable();
        Ok(Value::List(list))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_no_args() {
        assert_eq!(
            SortFilter.filter_list(vec!["item".into()], &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
    }

    #[test]
    fn it_sorts_lists() -> Result<(), FilterError> {
        let filter = SortFilter;

        assert_eq!(
            filter.filter_list(vec!["c".into(), "a".into(), "c".into()], &[])?,
            Value::List(vec!["a".into(), "c".into(), "c".into()])
        );

        Ok(())
    }
}
