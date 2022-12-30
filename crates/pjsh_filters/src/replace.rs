use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter that replaces values.
///
/// For lists, the filter replaces entire words.
///
/// For words, the filter replaces character patterns.
#[derive(Debug, Clone)]
pub struct ReplaceFilter;
impl Filter for ReplaceFilter {
    fn name(&self) -> &str {
        "replace"
    }

    fn filter_list(&self, list: Vec<String>, args: &[String]) -> FilterResult {
        let (from, to) = match &args {
            [] => return Err(FilterError::MissingArg("from")),
            [_] => return Err(FilterError::MissingArg("to")),
            [from, to] => (from, to),
            _ => return Err(FilterError::TooManyArgs),
        };

        let list = list
            .into_iter()
            .map(|item| if &item == from { to.to_string() } else { item })
            .collect();

        Ok(Value::List(list))
    }

    fn filter_word(&self, word: String, args: &[String]) -> FilterResult {
        let (from, to) = match &args {
            [] => return Err(FilterError::MissingArg("from")),
            [_] => return Err(FilterError::MissingArg("to")),
            [from, to] => (from, to),
            _ => return Err(FilterError::TooManyArgs),
        };

        Ok(Value::Word(word.replace(from, to)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_two_args() {
        let filter = ReplaceFilter;
        assert_eq!(
            filter.filter_list(vec!["item".into()], &[]),
            Err(FilterError::MissingArg("from"))
        );
        assert_eq!(
            filter.filter_list(vec!["item".into()], &["from".into()]),
            Err(FilterError::MissingArg("to"))
        );
        assert_eq!(
            filter.filter_list(
                vec!["item".into()],
                &["from".into(), "to".into(), "extra".into()]
            ),
            Err(FilterError::TooManyArgs)
        );

        assert_eq!(
            filter.filter_word("word".into(), &[]),
            Err(FilterError::MissingArg("from"))
        );
        assert_eq!(
            filter.filter_word("word".into(), &["from".into()]),
            Err(FilterError::MissingArg("to"))
        );
        assert_eq!(
            filter.filter_word("word".into(), &["from".into(), "to".into(), "extra".into()]),
            Err(FilterError::TooManyArgs)
        );
    }

    #[test]
    fn it_replaces_list_items() -> Result<(), FilterError> {
        let filter = ReplaceFilter;

        assert_eq!(
            filter.filter_list(
                vec!["a".into(), "b".into(), "c".into()],
                &["a".into(), "b".into()]
            )?,
            Value::List(vec!["b".into(), "b".into(), "c".into()])
        );

        Ok(())
    }

    #[test]
    fn it_replaces_word_chars() -> Result<(), FilterError> {
        let filter = ReplaceFilter;

        assert_eq!(
            filter.filter_word("abc".into(), &["b".into(), "c".into()])?,
            Value::Word("acc".into()),
        );

        Ok(())
    }
}
