use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use parking_lot::Mutex;
use pjsh_ast::Word;
use pjsh_core::{utils::path_to_string, Context, Scope};
use sysinfo::{get_current_pid, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};

use crate::{executor::execute_program, Executor};

pub fn interpolate_word(executor: &Executor, word: Word, context: Arc<Mutex<Context>>) -> String {
    match word {
        Word::Literal(literal) => literal,
        Word::Quoted(quoted) => quoted,
        Word::Variable(name) => interpolate_variable(&name, &context.lock()),
        Word::Interpolation(units) => {
            let mut output = String::new();

            for unit in units {
                match unit {
                    pjsh_ast::InterpolationUnit::Literal(literal) => output.push_str(&literal),
                    pjsh_ast::InterpolationUnit::Unicode(ch) => output.push(ch),
                    pjsh_ast::InterpolationUnit::Variable(name) => {
                        output.push_str(&interpolate_variable(&name, &context.lock()));
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

fn interpolate_variable(name: &str, context: &Context) -> String {
    // Interpolate positional arguments.
    if let Ok(i) = name.parse::<usize>() {
        return context
            .args()
            .get(i)
            .map(String::to_owned)
            .unwrap_or_default();
    }

    // Interpolate shell-reserved variables.
    if let Some(value) = eval_special_variable(name, context) {
        return value;
    }

    // Resolve variables.
    context.get_var(name).unwrap_or_default().to_owned()
}

fn eval_special_variable(key: &str, context: &Context) -> Option<String> {
    match key {
        "$" => Some(context.host.lock().process_id().to_string()),
        "?" => Some(context.last_exit.to_string()),
        "HOME" => dirs::home_dir().map(|p| path_to_string(&p)),
        "PPID" => {
            if let Ok(pid) = get_current_pid() {
                let system = System::new_with_specifics(
                    RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
                );
                if let Some(process) = system.process(pid) {
                    if let Some(parent_id) = process.parent() {
                        return Some(parent_id.to_string());
                    }
                }
            }

            None
        }
        "PWD" => std::env::current_dir().map(|p| path_to_string(&p)).ok(),
        "SHELL" => std::env::current_exe().map(|p| path_to_string(&p)).ok(),
        _ => None,
    }
}
