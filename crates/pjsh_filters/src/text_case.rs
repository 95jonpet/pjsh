use pjsh_core::{Filter, FilterError, FilterResult, Value};

/// A filter that converts words into lowercase.
#[derive(Debug, Clone)]
pub struct LowercaseFilter;
impl Filter for LowercaseFilter {
    fn name(&self) -> &str {
        "lowercase"
    }

    fn filter_word(&self, word: String, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        Ok(Value::Word(word.to_lowercase()))
    }
}

/// A filter that converts words into uppercase.
#[derive(Debug, Clone)]
pub struct UppercaseFilter;
impl Filter for UppercaseFilter {
    fn name(&self) -> &str {
        "uppercase"
    }

    fn filter_word(&self, word: String, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        Ok(Value::Word(word.to_uppercase()))
    }
}

/// A filter that converts the first letter of words into uppercase.
#[derive(Debug, Clone)]
pub struct UcfirstFilter;
impl Filter for UcfirstFilter {
    fn name(&self) -> &str {
        "ucfirst"
    }

    fn filter_word(&self, word: String, args: &[String]) -> FilterResult {
        if !args.is_empty() {
            return Err(FilterError::NoArgsAllowed);
        }

        let mut chars = word.chars();
        let Some(first) = chars.next() else {
            return Ok(Value::Word(String::new()));
        };

        let word = first.to_uppercase().to_string() + chars.as_str();
        Ok(Value::Word(word))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_accepts_no_args() {
        assert_eq!(
            LowercaseFilter.filter_word("word".into(), &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
        assert_eq!(
            UcfirstFilter.filter_word("word".into(), &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
        assert_eq!(
            UppercaseFilter.filter_word("word".into(), &["not-allowed".into()]),
            Err(FilterError::NoArgsAllowed)
        );
    }

    #[test]
    fn it_converts_chars_to_lowercase() -> Result<(), FilterError> {
        assert_eq!(
            LowercaseFilter.filter_word("aBcÅäÖ".into(), &[])?,
            Value::Word("abcåäö".into()),
        );

        Ok(())
    }

    #[test]
    fn it_converts_chars_to_uppercase() -> Result<(), FilterError> {
        assert_eq!(
            UppercaseFilter.filter_word("aBcÅäÖ".into(), &[])?,
            Value::Word("ABCÅÄÖ".into()),
        );

        Ok(())
    }

    #[test]
    fn it_converts_first_char_to_uppercase() -> Result<(), FilterError> {
        assert_eq!(
            UcfirstFilter.filter_word("".into(), &[])?,
            Value::Word("".into()),
        );
        assert_eq!(
            UcfirstFilter.filter_word("abc".into(), &[])?,
            Value::Word("Abc".into()),
        );
        assert_eq!(
            UcfirstFilter.filter_word("åäö".into(), &[])?,
            Value::Word("Åäö".into()),
        );

        Ok(())
    }
}
