use std::sync::Arc;

use pjsh_ast::{
    AndOr, Assignment, Command, ConditionalChain, Pipeline, PipelineSegment, Statement, Word,
};
use pjsh_core::Context;

use crate::{tests::utils::test_executor, FileDescriptors};

fn pipeline(exit_status: i32) -> Pipeline {
    Pipeline {
        is_async: false,
        segments: vec![PipelineSegment::Command(Command {
            program: Word::Literal("exit".into()),
            arguments: vec![Word::Literal(format!("{}", exit_status))],
            redirects: Vec::new(),
        })],
    }
}

#[test]
fn execute_assign() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let assignment = Assignment::new(Word::Literal("key".into()), Word::Literal("value".into()));

    executor.execute_statement(Statement::Assignment(assignment), Arc::clone(&ctx), &fds);

    assert_eq!(ctx.lock().scope.get_env("key"), Some("value".into()));
}

#[test]
fn execute_assign_replace() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let assignment = Assignment::new(Word::Literal("key".into()), Word::Literal("new".into()));

    ctx.lock().scope.set_env("key".into(), "old".into());
    executor.execute_statement(Statement::Assignment(assignment), Arc::clone(&ctx), &fds);

    assert_eq!(ctx.lock().scope.get_env("key"), Some("new".into()));
}

#[test]
fn execute_if_statement_true() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let conditional = ConditionalChain {
        conditions: vec![AndOr {
            operators: Vec::new(),
            pipelines: vec![pipeline(0)], // 0 = success -> true
        }],
        branches: vec![pjsh_ast::Program {
            statements: vec![Statement::Assignment(Assignment::new(
                Word::Literal("key".into()),
                Word::Literal("new".into()),
            ))],
        }],
    };

    ctx.lock().scope.set_env("key".into(), "old".into());
    executor.execute_statement(Statement::If(conditional), Arc::clone(&ctx), &fds);

    assert_eq!(
        ctx.lock().scope.get_env("key"),
        Some("new".into()),
        "the branch is taken"
    );
}

#[test]
fn execute_if_statement_false() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let conditional = ConditionalChain {
        conditions: vec![AndOr {
            operators: Vec::new(),
            pipelines: vec![pipeline(1)], // 1 != success -> false
        }],
        branches: vec![pjsh_ast::Program {
            statements: vec![Statement::Assignment(Assignment::new(
                Word::Literal("key".into()),
                Word::Literal("new".into()),
            ))],
        }],
    };

    ctx.lock().scope.set_env("key".into(), "old".into());
    executor.execute_statement(Statement::If(conditional), Arc::clone(&ctx), &fds);

    assert_eq!(ctx.lock().last_exit, 0, "should always exit with 0");
    assert_eq!(
        ctx.lock().scope.get_env("key"),
        Some("old".into()),
        "the branch is not taken"
    );
}

#[test]
fn execute_if_statement_second_branch() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let conditional = ConditionalChain {
        conditions: vec![
            AndOr {
                operators: Vec::new(),
                pipelines: vec![pipeline(1)], // 1 != success -> false
            },
            AndOr {
                operators: Vec::new(),
                pipelines: vec![pipeline(0)], // 0 = success -> true
            },
        ],
        branches: vec![
            pjsh_ast::Program {
                statements: vec![Statement::Assignment(Assignment::new(
                    Word::Literal("key".into()),
                    Word::Literal("first".into()),
                ))],
            },
            pjsh_ast::Program {
                statements: vec![Statement::Assignment(Assignment::new(
                    Word::Literal("key".into()),
                    Word::Literal("second".into()),
                ))],
            },
            pjsh_ast::Program {
                statements: vec![Statement::Assignment(Assignment::new(
                    Word::Literal("key".into()),
                    Word::Literal("else".into()),
                ))],
            },
        ],
    };

    ctx.lock().scope.set_env("key".into(), "old".into());
    executor.execute_statement(Statement::If(conditional), Arc::clone(&ctx), &fds);

    assert_eq!(ctx.lock().last_exit, 0, "should always exit with 0");
    assert_eq!(
        ctx.lock().scope.get_env("key"),
        Some("second".into()),
        "the second branch is not taken"
    );
}

#[test]
fn execute_if_statement_else() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let conditional = ConditionalChain {
        conditions: vec![AndOr {
            operators: Vec::new(),
            pipelines: vec![pipeline(1)], // 1 != success -> false
        }],
        branches: vec![
            pjsh_ast::Program {
                statements: vec![Statement::Assignment(Assignment::new(
                    Word::Literal("key".into()),
                    Word::Literal("if".into()),
                ))],
            },
            pjsh_ast::Program {
                statements: vec![Statement::Assignment(Assignment::new(
                    Word::Literal("key".into()),
                    Word::Literal("else".into()),
                ))],
            },
        ],
    };

    ctx.lock().scope.set_env("key".into(), "old".into());
    executor.execute_statement(Statement::If(conditional), Arc::clone(&ctx), &fds);

    assert_eq!(ctx.lock().last_exit, 0, "should always exit with 0");
    assert_eq!(
        ctx.lock().scope.get_env("key"),
        Some("else".into()),
        "the else branch is not taken"
    );
}
