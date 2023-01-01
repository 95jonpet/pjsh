use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_ast::Program;
use pjsh_core::Context;
use pjsh_eval::execute_statement;

/// Trait for implementing program execution.
pub(crate) trait Execute {
    /// Executes a program within a context.
    fn execute(&self, program: Program, context: Arc<Mutex<Context>>);
}

/// Program executor.
pub(crate) struct ProgramExecutor {
    /// Exit on encountering errors during execution.
    exit_on_error: bool,
}

impl ProgramExecutor {
    /// Constructs a new program executor.
    pub(crate) fn new(exit_on_error: bool) -> Self {
        Self { exit_on_error }
    }
}

impl Execute for ProgramExecutor {
    fn execute(&self, program: Program, context: Arc<Mutex<Context>>) {
        for statement in program.statements {
            let Err(error) = execute_statement(&statement, &mut context.lock()) else {
                continue;
            };

            eprintln!("pjsh: {error}");

            // Ensure that a non-0 exit code is set.
            if context.lock().last_exit() == 0 {
                context.lock().register_exit(1);
            }

            if self.exit_on_error {
                break;
            }
        }
    }
}

/// Prints a parsed AST to stdout.
pub(crate) struct AstPrinter;

impl Execute for AstPrinter {
    fn execute(&self, program: Program, _context: Arc<Mutex<Context>>) {
        for statement in program.statements {
            println!("{:#?}", statement);
        }
    }
}
