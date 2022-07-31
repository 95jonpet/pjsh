use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_ast::Program;
use pjsh_core::Context;
use pjsh_exec::{Executor, FileDescriptors};

/// Trait for implementing program execution.
pub(crate) trait Execute {
    /// Executes a program within a context.
    fn execute(&self, program: Program, context: Arc<Mutex<Context>>);
}

/// Program executor.
pub(crate) struct ProgramExecutor {
    /// Internal executor.
    executor: Executor,
}

impl ProgramExecutor {
    /// Constructs a new program executor.
    pub fn new() -> Self {
        Self {
            executor: create_executor(),
        }
    }
}

impl Execute for ProgramExecutor {
    fn execute(&self, program: Program, context: Arc<Mutex<Context>>) {
        for statement in program.statements {
            let fds = FileDescriptors::new();
            self.executor
                .execute_statement(statement, Arc::clone(&context), &fds);
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

/// Creates a new executor with registered built-in commands.
pub(crate) fn create_executor() -> Executor {
    Executor::new(vec![
        Box::new(pjsh_builtins::Alias),
        Box::new(pjsh_builtins::Cd),
        Box::new(pjsh_builtins::Echo),
        Box::new(pjsh_builtins::Exit),
        Box::new(pjsh_builtins::False),
        Box::new(pjsh_builtins::Interpolate),
        Box::new(pjsh_builtins::Pwd),
        Box::new(pjsh_builtins::Sleep),
        Box::new(pjsh_builtins::Source),
        Box::new(pjsh_builtins::True),
        Box::new(pjsh_builtins::Type),
        Box::new(pjsh_builtins::Unalias),
        Box::new(pjsh_builtins::Unset),
        Box::new(pjsh_builtins::Which),
    ])
}
