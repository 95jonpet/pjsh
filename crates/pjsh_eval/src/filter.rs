use itertools::Itertools;
use pjsh_ast::Filter;
use pjsh_core::{Context, Value};

use crate::{interpolate_word, EvalError, EvalResult};

/// Returns the result of applying a filter to a value.
pub fn apply_filter(filter: &Filter, value: &Value, context: &Context) -> EvalResult<Value> {
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
        (Filter::Reverse, Value::List(items)) => {
            Ok(Value::List(items.clone().into_iter().rev().collect()))
        }
        (Filter::Sort, Value::List(items)) => {
            Ok(Value::List(items.clone().into_iter().sorted().collect()))
        }
        (Filter::Split(sep), Value::Word(word)) => Ok(Value::List(
            word.split(&interpolate_word(sep, context)?)
                .map(ToString::to_string)
                .collect(),
        )),
        (Filter::Unique, Value::List(items)) => {
            Ok(Value::List(items.clone().into_iter().unique().collect()))
        }
        (filter, value) => {
            let value_type = value_type_name(value);
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
