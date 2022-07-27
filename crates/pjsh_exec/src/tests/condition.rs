use std::sync::Arc;

use pjsh_ast::{AndOr, Pipeline, PipelineSegment, Word};
use pjsh_core::Context;

use crate::{tests::utils::test_executor, FileDescriptors};

fn pipeline(condition: &[&str]) -> Pipeline {
    let input = condition
        .iter()
        .map(|word| Word::Literal(word.to_string()))
        .collect();

    Pipeline {
        is_async: false,
        segments: vec![PipelineSegment::Condition(input)],
    }
}

#[test]
fn true_condition() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_or = AndOr {
        operators: Vec::new(),
        pipelines: vec![pipeline(&["true", "==", "true"])],
    };
    executor.execute_and_or(and_or, Arc::clone(&ctx), &fds);
    assert_eq!(ctx.lock().last_exit(), 0);
}

#[test]
fn false_condition() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_or = AndOr {
        operators: Vec::new(),
        pipelines: vec![pipeline(&["true", "==", "false"])],
    };
    executor.execute_and_or(and_or, Arc::clone(&ctx), &fds);
    assert_eq!(ctx.lock().last_exit(), 1);
}

#[test]
fn inverted_condition() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_or = AndOr {
        operators: Vec::new(),
        pipelines: vec![pipeline(&["!", "true", "==", "false"])],
    };
    executor.execute_and_or(and_or, Arc::clone(&ctx), &fds);
    assert_eq!(ctx.lock().last_exit(), 0); // !false == true
}

#[test]
fn invalid_condition() {
    let fds = FileDescriptors::new();
    let ctx = Arc::new(parking_lot::Mutex::new(Context::default()));
    let executor = test_executor();
    let and_or = AndOr {
        operators: Vec::new(),
        pipelines: vec![pipeline(&["invalid", "condition"])],
    };
    executor.execute_and_or(and_or, Arc::clone(&ctx), &fds);
    assert_ne!(ctx.lock().last_exit(), 0);
}
