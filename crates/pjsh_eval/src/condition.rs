use std::path::PathBuf;

use pjsh_ast::{Condition, Word};
use pjsh_core::{utils::resolve_path, Context};

use crate::{error::EvalResult, interpolate_word};

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
    fn test_invert() {
        let boxed_true = Box::new(Condition::Empty(Word::Literal(String::new())));
        let boxed_false = Box::new(Condition::Empty(Word::Literal("non-empty".into())));

        assert!(!eval_condition(&Condition::Invert(boxed_true), &Context::default()).unwrap());
        assert!(eval_condition(&Condition::Invert(boxed_false), &Context::default()).unwrap());
    }
}
