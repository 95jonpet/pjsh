use pjsh_ast::Filter;
use pjsh_core::{Context, Value};

use crate::{interpolate_word, EvalError, EvalResult};

/// Returns the result of applying a filter to a value.
pub(crate) fn apply_filter(
    ast_filter: &Filter,
    value: Value,
    context: &Context,
) -> EvalResult<Value> {
    // Get the registered filter with a matching name.
    let filter_name = interpolate_word(&ast_filter.name, context)?;
    let Some(filter) = context.filters.get(&filter_name) else {
        return Err(EvalError::UnknownFilter(filter_name));
    };

    // Resolve arguments after matching the filter.
    let mut args = Vec::with_capacity(ast_filter.args.len());
    for arg in &ast_filter.args {
        args.push(interpolate_word(arg, context)?);
    }

    // Apply the filter.
    let result = match value {
        Value::Word(word) => filter.filter_word(word, &args[..]),
        Value::List(list) => filter.filter_list(list, &args[..]),
    };

    result.map_err(|error| EvalError::FilterError(filter_name, error))
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use pjsh_ast::Word;
    use pjsh_core::{Filter, FilterResult};

    use super::*;

    #[test]
    fn it_errors_on_unknown_filters() {
        let unknown_filter = pjsh_ast::Filter {
            name: Word::Literal("unknown".into()),
            args: vec![],
        };
        assert!(matches!(
            apply_filter(
                &unknown_filter,
                Value::Word("word".into()),
                &Context::default(),
            ),
            Err(EvalError::UnknownFilter(name)) if name == "unknown"
        ));
    }

    #[test]
    fn it_applies_filters_to_lists() -> EvalResult<()> {
        #[derive(Clone)]
        struct ListFilter {
            counter: Rc<RefCell<usize>>,
        }

        impl Filter for ListFilter {
            fn name(&self) -> &str {
                "listfilter"
            }

            fn filter_list(&self, list: Vec<String>, _args: &[String]) -> FilterResult {
                *self.counter.borrow_mut() += 1;
                Ok(Value::List(list))
            }
        }

        let counter = Rc::new(RefCell::new(0));
        let filter = ListFilter {
            counter: Rc::clone(&counter),
        };
        let mut ctx = Context::default();
        ctx.filters.insert(filter.name().into(), Box::new(filter));

        let ast_filter = pjsh_ast::Filter {
            name: Word::Literal("listfilter".into()),
            args: vec![Word::Literal("arg".into())],
        };

        apply_filter(&ast_filter, Value::List(vec!["item".into()]), &ctx)?;

        assert!(*counter.borrow() == 1, "the filter should be applied");

        Ok(())
    }

    #[test]
    fn it_applies_filters_to_words() -> EvalResult<()> {
        #[derive(Clone)]
        struct WordFilter {
            counter: Rc<RefCell<usize>>,
        }

        impl Filter for WordFilter {
            fn name(&self) -> &str {
                "wordfilter"
            }

            fn filter_word(&self, word: String, _args: &[String]) -> FilterResult {
                *self.counter.borrow_mut() += 1;
                Ok(Value::Word(word))
            }
        }

        let counter = Rc::new(RefCell::new(0));
        let filter = WordFilter {
            counter: Rc::clone(&counter),
        };
        let mut ctx = Context::default();
        ctx.filters.insert(filter.name().into(), Box::new(filter));

        let ast_filter = pjsh_ast::Filter {
            name: Word::Literal("wordfilter".into()),
            args: vec![Word::Literal("arg".into())],
        };

        apply_filter(&ast_filter, Value::Word("word".into()), &ctx)?;

        assert!(*counter.borrow() == 1, "the filter should be applied");

        Ok(())
    }
}
