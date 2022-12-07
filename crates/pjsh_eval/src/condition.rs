use std::path::PathBuf;

use pjsh_ast::{Condition, Word};
use pjsh_core::{Context, utils::resolve_path};

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
