mod fs;
mod logic;
mod string;

use pjsh_core::{utils::resolve_path, Condition, Context};

use crate::error::ExecError;

/// Parses a [`Condition`].
pub(crate) fn parse_condition(
    input: &[&str],
    ctx: &Context,
) -> Result<Box<dyn Condition>, ExecError> {
    // The "!" symbol can be used to invert a condition.
    if matches!(input.first(), Some(&"!")) {
        return Ok(Box::new(logic::Invert(parse_condition(&input[1..], ctx)?)));
    }

    match input {
        // String-related conditions.
        [a, "!=", b] => Ok(Box::new(string::NotEqual(a.to_string(), b.to_string()))),
        [a, "==", b] => Ok(Box::new(string::Equal(a.to_string(), b.to_string()))),
        [a, "=", b] => Ok(Box::new(string::Equal(a.to_string(), b.to_string()))),
        ["-z", string] => Ok(Box::new(string::Empty(string.to_string()))),
        ["-n", string] => Ok(Box::new(string::NotEmpty(string.to_string()))),
        [string] => Ok(Box::new(string::NotEmpty(string.to_string()))),

        // File-related conditions.
        ["-e", path] => Ok(Box::new(fs::Exists(resolve_path(ctx, path)))),
        ["is-path", path] => Ok(Box::new(fs::Exists(resolve_path(ctx, path)))),
        ["-f", path] => Ok(Box::new(fs::IsFile(resolve_path(ctx, path)))),
        ["is-file", path] => Ok(Box::new(fs::IsFile(resolve_path(ctx, path)))),
        ["-d", path] => Ok(Box::new(fs::IsDirectory(resolve_path(ctx, path)))),
        ["is-dir", path] => Ok(Box::new(fs::IsDirectory(resolve_path(ctx, path)))),

        // Undefined conditions are considered invalid.
        _ => {
            let mut text = String::new();
            let mut words = input.iter();

            if let Some(word) = words.next() {
                text.push_str(word.as_ref());
            }

            for word in words {
                text.push(' ');
                text.push_str(word.as_ref());
            }

            Err(ExecError::Message(format!("invalid condition: {text}")))
        }
    }
}
