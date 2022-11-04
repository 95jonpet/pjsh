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
pub(crate) struct ProgramExecutor;

impl Execute for ProgramExecutor {
    fn execute(&self, program: Program, context: Arc<Mutex<Context>>) {
        for statement in program.statements {
            if let Err(error) = execute_statement(&statement, &mut context.lock()) {
                eprintln!("pjsh: {error}");
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
