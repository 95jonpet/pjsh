use std::sync::Arc;

use pjsh_ast::{Assignment, Statement, Word};
use pjsh_core::Context;

use crate::Executor;

#[test]
fn execute_assign() {
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = Executor;
    let assignment = Assignment::new(Word::Literal("key".into()), Word::Literal("value".into()));

    executor.execute_statement(Statement::Assignment(assignment), Arc::clone(&ctx));

    assert_eq!(ctx.lock().scope.get_env("key"), Some("value".into()));
}

#[test]
fn execute_assign_replace() {
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = Executor;
    let assignment = Assignment::new(Word::Literal("key".into()), Word::Literal("new".into()));

    ctx.lock().scope.set_env("key".into(), "old".into());
    executor.execute_statement(Statement::Assignment(assignment), Arc::clone(&ctx));

    assert_eq!(ctx.lock().scope.get_env("key"), Some("new".into()));
}
