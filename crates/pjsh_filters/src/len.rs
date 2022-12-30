use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter for returning the length of a list.
#[derive(Debug, Clone)]
pub struct LenFilter;
impl Filter for LenFilter {
    fn name(&self) -> &str {
        "len"
    }

    fn filter_list(&self, list: Vec<String>, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        Ok(Value::Word(list.len().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_no_args() {
        let filter = LenFilter;
        assert_eq!(
            filter.filter_list(vec![], &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
    }

    #[test]
    fn it_returns_list_len() -> Result<(), FilterError> {
        let filter = LenFilter;

        assert_eq!(filter.filter_list(vec![], &[])?, Value::Word(0.to_string()));
        assert_eq!(
            filter.filter_list(vec!["1".into()], &[])?,
            Value::Word(1.to_string())
        );
        assert_eq!(
            filter.filter_list(vec!["1".into(), "2".into()], &[])?,
            Value::Word(2.to_string())
        );

        Ok(())
    }
}
