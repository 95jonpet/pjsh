use itertools::Itertools;
use pjsh_ast::Filter;
use pjsh_core::{Context, Value};
use pjsh_parse::is_whitespace;

use crate::{interpolate_word, EvalError, EvalResult};

/// Returns the result of applying a filter to a value.
pub(crate) fn apply_filter(filter: &Filter, value: Value, context: &Context) -> EvalResult<Value> {
    match (filter, value) {
        (Filter::Index(index), Value::List(list)) => {
            let Ok(index) = interpolate_word(index, context)?.parse::<usize>() else {
                return Err(EvalError::InvalidIndex);
            };

            list.get(index)
                .cloned()
                .ok_or(EvalError::InvalidIndex)
                .map(Value::Word)
        }
        (Filter::Join(sep), Value::List(list)) => {
            Ok(Value::Word(list.join(&interpolate_word(sep, context)?)))
        }
        (Filter::Len, Value::List(list)) => Ok(Value::Word(list.len().to_string())),
        (Filter::Lower, Value::Word(word)) => Ok(Value::Word(word.to_lowercase())),
        (Filter::Upper, Value::Word(word)) => Ok(Value::Word(word.to_uppercase())),
        (Filter::Words, Value::Word(word)) => Ok(Value::List(
            word.split(is_whitespace)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect(),
        )),
        (Filter::Reverse, Value::List(mut items)) => {
            items.reverse();
            Ok(Value::List(items))
        }
        (Filter::Sort, Value::List(mut items)) => {
            items.sort();
            Ok(Value::List(items))
        }
        (Filter::Split(sep), Value::Word(word)) => Ok(Value::List(
            word.split(&interpolate_word(sep, context)?)
                .map(ToString::to_string)
                .collect(),
        )),
        (Filter::Unique, Value::List(items)) => {
            Ok(Value::List(items.into_iter().unique().collect()))
        }
        (Filter::Replace(from, to), Value::Word(word)) => {
            let from = interpolate_word(from, context)?;
            let to = interpolate_word(to, context)?;
            Ok(Value::Word(word.replace(&from, &to)))
        }
        (Filter::Replace(from, to), Value::List(items)) => {
            let from = interpolate_word(from, context)?;
            let to = interpolate_word(to, context)?;
            let items = items
                .into_iter()
                .map(|item| if item == from { to.clone() } else { item })
                .collect();
            Ok(Value::List(items))
        }
        (filter, value) => {
            let value_type = value_type_name(&value);
            let message = format!("can't apply filter {filter} to value of type {value_type}");
            Err(EvalError::InvalidValuePipeline(message))
        }
    }
}

/// Returns a value type's name.
fn value_type_name(value: &Value) -> &str {
    match value {
        Value::Word(_) => "word",
        Value::List(_) => "list",
    }
}
