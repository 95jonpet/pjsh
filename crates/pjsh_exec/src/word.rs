use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use parking_lot::Mutex;
use pjsh_ast::Word;
use pjsh_core::{Context, Scope};

use crate::{executor::execute_program, Executor};

pub fn interpolate_word(executor: &Executor, word: Word, context: Arc<Mutex<Context>>) -> String {
    match word {
        Word::Literal(literal) => literal,
        Word::Quoted(quoted) => quoted,
        Word::Variable(key) => match key.as_str() {
            "$" => context.lock().host.lock().process_id().to_string(),
            "?" => context.lock().last_exit.to_string(),
            _ => {
                if let Ok(positional) = key.parse::<usize>() {
                    return context
                        .lock()
                        .args()
                        .get(positional)
                        .map(String::to_owned)
                        .unwrap_or_else(String::new);
                }

                context.lock().get_var(&key).unwrap_or_default().to_owned()
            }
        },
        Word::Interpolation(units) => {
            let mut output = String::new();

            for unit in units {
                match unit {
                    pjsh_ast::InterpolationUnit::Literal(literal) => output.push_str(&literal),
                    pjsh_ast::InterpolationUnit::Unicode(ch) => output.push(ch),
                    pjsh_ast::InterpolationUnit::Variable(variable) => {
                        output.push_str(context.lock().get_var(&variable).unwrap_or_default())
                    }
                    pjsh_ast::InterpolationUnit::Subshell(program) => {
                        context.lock().push_scope(Scope::new(
                            format!("{} subshell", context.lock().name()),
                            Vec::default(),
                            HashMap::default(),
                            HashMap::default(),
                            HashSet::default(),
                            false,
                        ));
                        let inner_context = Arc::clone(&context);
                        output.push_str(&execute_program(executor, program, inner_context).0);
                        context.lock().pop_scope();
                    }
                }
            }

            output
        }
        Word::Subshell(program) => {
            context.lock().push_scope(Scope::new(
                format!("{} subshell", context.lock().name()),
                Vec::default(),
                HashMap::default(),
                HashMap::default(),
                HashSet::default(),
                false,
            ));
            let inner_context = Arc::clone(&context);
            let interpolation = execute_program(executor, program, inner_context).0;
            context.lock().pop_scope();
            interpolation
        }
    }
}
