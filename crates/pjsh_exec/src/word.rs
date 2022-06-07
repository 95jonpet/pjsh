use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_ast::Word;
use pjsh_core::Context;

use crate::{executor::execute_program, Executor};

pub fn interpolate_word(executor: &Executor, word: Word, context: &Context) -> String {
    match word {
        Word::Literal(literal) => literal,
        Word::Quoted(quoted) => quoted,
        Word::Variable(key) => match key.as_str() {
            "$" => context.host.lock().process_id().to_string(),
            "?" => context.last_exit.to_string(),
            _ => {
                if let Ok(positional) = key.parse::<usize>() {
                    return context
                        .arguments
                        .get(positional)
                        .map(String::to_owned)
                        .unwrap_or_else(String::new);
                }

                context.scope.get_env(&key).unwrap_or_default()
            }
        },
        Word::Interpolation(units) => {
            let mut output = String::new();

            for unit in units {
                match unit {
                    pjsh_ast::InterpolationUnit::Literal(literal) => output.push_str(&literal),
                    pjsh_ast::InterpolationUnit::Unicode(ch) => output.push(ch),
                    pjsh_ast::InterpolationUnit::Variable(variable) => {
                        output.push_str(&context.scope.get_env(&variable).unwrap_or_default())
                    }
                    pjsh_ast::InterpolationUnit::Subshell(program) => {
                        let inner_context =
                            Arc::new(Mutex::new(context.fork(context.name.clone())));
                        output.push_str(&execute_program(executor, program, inner_context).0)
                    }
                }
            }

            output
        }
        Word::Subshell(program) => {
            let inner_context = Arc::new(Mutex::new(context.fork(context.name.clone())));
            execute_program(executor, program, inner_context).0
        }
    }
}
