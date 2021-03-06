use std::sync::Arc;

use pjsh_ast::{AndOr, AndOrOp, Command, Pipeline, PipelineSegment, Word};
use pjsh_core::Context;

use crate::{tests::utils::test_executor, FileDescriptors};

fn pipeline(exit_status: i32) -> Pipeline {
    Pipeline {
        is_async: false,
        segments: vec![PipelineSegment::Command(Command {
            arguments: vec![
                Word::Literal("exit".into()),
                Word::Literal(format!("{}", exit_status)),
            ],
            redirects: Vec::new(),
        })],
    }
}

#[test]
fn execute_and_success() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_success = AndOr {
        operators: vec![AndOrOp::And],
        pipelines: vec![pipeline(0), pipeline(0)],
    };
    executor.execute_and_or(and_success, Arc::clone(&ctx), &fds);
    assert_eq!(ctx.lock().last_exit, 0);
}

#[test]
fn execute_and_fail() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_success = AndOr {
        operators: vec![AndOrOp::And],
        pipelines: vec![pipeline(1), pipeline(0)],
    };
    executor.execute_and_or(and_success, Arc::clone(&ctx), &fds);
    assert_eq!(ctx.lock().last_exit, 1);
}

#[test]
fn execute_or_first_success() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_success = AndOr {
        operators: vec![AndOrOp::Or],
        pipelines: vec![pipeline(0), pipeline(1)],
    };
    executor.execute_and_or(and_success, Arc::clone(&ctx), &fds);
    assert_eq!(ctx.lock().last_exit, 0);
}

#[test]
fn execute_or_last_success() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_success = AndOr {
        operators: vec![AndOrOp::Or],
        pipelines: vec![pipeline(1), pipeline(0)],
    };
    executor.execute_and_or(and_success, Arc::clone(&ctx), &fds);
    assert_eq!(ctx.lock().last_exit, 0);
}

#[test]
fn execute_or_last_fail() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_success = AndOr {
        operators: vec![AndOrOp::Or],
        pipelines: vec![pipeline(1), pipeline(1)],
    };
    executor.execute_and_or(and_success, Arc::clone(&ctx), &fds);
    assert_eq!(ctx.lock().last_exit, 1);
}
