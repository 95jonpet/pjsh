use std::path::PathBuf;

use pjsh_ast::{Condition, Word};
use pjsh_core::{utils::resolve_path, Context};
use regex::RegexBuilder;

use crate::{error::EvalResult, interpolate_word, EvalError};

/// Size limit, in bytes, for regular expressions.
const REGEX_SIZE_LIMIT: usize = 4096; // TODO: Set the regex size limit to a sensible default value.

/// Evaluates a condition within a context.
///
/// # Errors
///
/// This function will return an error if the condition cannot be evaluated.
pub fn eval_condition(condition: &Condition, context: &Context) -> EvalResult<bool> {
    match condition {
        Condition::IsDirectory(path) => if_path(path, context, |p| p.is_dir()),
        Condition::IsFile(path) => if_path(path, context, |p| p.is_file()),
        Condition::IsPath(path) => if_path(path, context, |p| p.exists()),
        Condition::Empty(word) => Ok(interpolate_word(word, context)?.is_empty()),
        Condition::NotEmpty(word) => Ok(!interpolate_word(word, context)?.is_empty()),
        Condition::Eq(a, b) => if_compare(a, b, context, |a, b| a == b),
        Condition::Ne(a, b) => if_compare(a, b, context, |a, b| a != b),
        Condition::Matches(word, pattern) => matches_regex(word, pattern, context),
        Condition::Invert(condition) => Ok(!(eval_condition(condition, context)?)),
    }
}

/// Returns the result of a boolean function after interpolating two words.
///
/// # Errors
///
/// This function will return an error if any of the given words cannot be
/// interpolated.
fn if_compare<F: Fn(String, String) -> bool>(
    a: &Word,
    b: &Word,
    context: &Context,
    func: F,
) -> EvalResult<bool> {
    let a = interpolate_word(a, context)?;
    let b = interpolate_word(b, context)?;
    Ok(func(a, b))
}

/// Returns the result of a boolean function after interpolating a words and
/// converting it into a path.
///
/// # Errors
///
/// This function will return an error if the given word cannot be interpolated.
fn if_path<F: Fn(PathBuf) -> bool>(path: &Word, context: &Context, func: F) -> EvalResult<bool> {
    let path = resolve_path(context, interpolate_word(path, context)?);
    Ok(func(path))
}

/// Returns `true` if a word matches a regex pattern.
///
/// # Errors
///
/// This function will return an error if the word, or pattern, cannot be
/// interpolated or if the given pattern is not a valid regex.
///
/// This function will also return an error if the compiled regex exceeds
/// the maximum allowed regex size imposed by the shell. This prevents trivial
/// denial-of-service attacks.
fn matches_regex(word: &Word, pattern: &Word, context: &Context) -> EvalResult<bool> {
    let word = interpolate_word(word, context)?;
    let pattern = interpolate_word(pattern, context)?;

    // Construct a regex from untrusted input.
    // The regex is limited in size in order to prevent trivial denial-of-service
    // attacks from badly formed regular expressions.
    // TODO: Allow the regex size limit to be customized.
    let regex = RegexBuilder::new(&pattern)
        .size_limit(REGEX_SIZE_LIMIT)
        .dfa_size_limit(REGEX_SIZE_LIMIT)
        .build();

    regex
        .map(|regex| regex.is_match(&word))
        .map_err(|err| EvalError::InvalidRegex(format!("{err}")))
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, NamedTempFile};

    use super::*;

    /// Creates a temporary file and a temporary directory, executing a function
    /// with word representations of the temporary paths.
    fn in_temp_fs<F: Fn(Word, Word)>(func: F) {
        let file = NamedTempFile::new().expect("Temporary file can be created");
        let dir = tempdir().expect("Temporary directory can be created");

        let file = Word::Literal(file.path().to_string_lossy().to_string());
        let dir = Word::Literal(dir.path().to_string_lossy().to_string());

        func(file, dir);
    }

    #[test]
    fn test_is_directory() {
        in_temp_fs(|file, dir| {
            assert!(!eval_condition(&Condition::IsDirectory(file), &Context::default()).unwrap());
            assert!(eval_condition(&Condition::IsDirectory(dir), &Context::default()).unwrap());
        });
    }

    #[test]
    fn test_is_file() {
        in_temp_fs(|file, dir| {
            assert!(eval_condition(&Condition::IsFile(file), &Context::default()).unwrap());
            assert!(!eval_condition(&Condition::IsFile(dir), &Context::default()).unwrap());
        });
    }

    #[test]
    fn test_is_path() {
        in_temp_fs(|file, dir| {
            assert!(eval_condition(&Condition::IsPath(file), &Context::default()).unwrap());
            assert!(eval_condition(&Condition::IsPath(dir), &Context::default()).unwrap());
        });
    }

    #[test]
    fn test_empty() {
        let empty = Word::Literal(String::new());
        let non_empty = Word::Literal("non-empty".into());
        assert!(eval_condition(&Condition::Empty(empty), &Context::default()).unwrap());
        assert!(!eval_condition(&Condition::Empty(non_empty), &Context::default()).unwrap());
    }

    #[test]
    fn test_not_empty() {
        let empty = Word::Literal(String::new());
        let non_empty = Word::Literal("non-empty".into());
        assert!(!eval_condition(&Condition::NotEmpty(empty), &Context::default()).unwrap());
        assert!(eval_condition(&Condition::NotEmpty(non_empty), &Context::default()).unwrap());
    }

    #[test]
    fn test_eq() {
        let a = Word::Literal("a".into());
        let b = Word::Literal("b".into());
        assert!(eval_condition(&Condition::Eq(a.clone(), a.clone()), &Context::default()).unwrap());
        assert!(!eval_condition(&Condition::Eq(a, b), &Context::default()).unwrap());
    }

    #[test]
    fn test_ne() {
        let a = Word::Literal("a".into());
        let b = Word::Literal("b".into());
        let context = Context::default();
        assert!(!eval_condition(&Condition::Ne(a.clone(), a.clone()), &context).unwrap());
        assert!(eval_condition(&Condition::Ne(a, b), &context).unwrap());
    }

    #[test]
    fn test_matches() {
        let a = Word::Literal("a".into());
        let b = Word::Literal("b".into());
        let pattern = Word::Literal("a+".into());

        let context = Context::default();
        assert!(eval_condition(&Condition::Matches(a, pattern.clone()), &context).unwrap());
        assert!(!eval_condition(&Condition::Matches(b, pattern), &context).unwrap());
    }

    #[test]
    fn test_matches_invalid_regex() {
        let a = Word::Literal("a".into());
        let pattern = Word::Literal("a{100}{100}{100}".into()); // Too large regex, prevent DoS.

        let context = Context::default();
        let result = eval_condition(&Condition::Matches(a, pattern), &context);

        assert!(matches!(result, Err(EvalError::InvalidRegex(_))));
    }

    #[test]
    fn test_invert() {
        let boxed_true = Box::new(Condition::Empty(Word::Literal(String::new())));
        let boxed_false = Box::new(Condition::Empty(Word::Literal("non-empty".into())));

        assert!(!eval_condition(&Condition::Invert(boxed_true), &Context::default()).unwrap());
        assert!(eval_condition(&Condition::Invert(boxed_false), &Context::default()).unwrap());
    }
}
