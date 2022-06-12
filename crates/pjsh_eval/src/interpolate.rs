use std::{collections::VecDeque, sync::Arc};

use parking_lot::Mutex;
use pjsh_ast::{InterpolationUnit, Word};
use pjsh_core::Context;

use crate::{EvalError, EvalResult};

pub(crate) fn interpolate(
    words: &[Word],
    ctx: Arc<Mutex<Context>>,
) -> EvalResult<VecDeque<String>> {
    words
        .iter()
        .map(|word| interpolate_word(word, Arc::clone(&ctx)))
        .collect()
}

pub(crate) fn interpolate_word(word: &Word, ctx: Arc<Mutex<Context>>) -> EvalResult<String> {
    match word {
        Word::Literal(content) | Word::Quoted(content) => Ok(content.to_owned()),
        Word::Variable(variable) => interpolate_variable(variable, ctx),
        Word::Subshell(_) => todo!("evaulate subshell"),
        Word::Interpolation(units) => interpolate_units(units, ctx),
        Word::ProcessSubstitution(_) => todo!("evaluate process substitution"),
    }
}

fn interpolate_units(units: &[InterpolationUnit], ctx: Arc<Mutex<Context>>) -> EvalResult<String> {
    let mut interpolation = String::new();
    for unit in units {
        interpolation.push_str(&interpolate_unit(unit, Arc::clone(&ctx))?)
    }
    Ok(interpolation)
}

fn interpolate_unit(unit: &InterpolationUnit, ctx: Arc<Mutex<Context>>) -> EvalResult<String> {
    match unit {
        InterpolationUnit::Literal(literal) => Ok(literal.to_owned()),
        InterpolationUnit::Unicode(ch) => Ok(ch.to_string()),
        InterpolationUnit::Variable(variable) => interpolate_variable(variable, ctx),
        InterpolationUnit::Subshell(_) => todo!("evaulate subshell"),
    }
}

pub(crate) fn interpolate_variable(variable: &str, ctx: Arc<Mutex<Context>>) -> EvalResult<String> {
    ctx.lock()
        .get_var(variable)
        .map(String::from)
        .ok_or_else(|| EvalError::UndefinedVariable(variable.to_string()))
}
