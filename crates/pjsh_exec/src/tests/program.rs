use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_ast::{
    AndOr, Command, FileDescriptor, Pipeline, PipelineSegment, Program, Redirect, Statement, Word,
};
use pjsh_core::Context;

#[test]
fn execute_program() {
    let ctx = Context::default();
    let program = Program {
        statements: vec![Statement::AndOr(AndOr {
            operators: Vec::new(),
            pipelines: vec![Pipeline {
                is_async: false,
                segments: vec![PipelineSegment {
                    command: Command {
                        program: Word::Literal("echo".into()),
                        arguments: vec![Word::Literal("Hello, world!".into())],
                        redirects: Vec::new(),
                    },
                }],
            }],
        })],
    };

    let (stdout, stderr) = crate::executor::execute_program(program, Arc::new(Mutex::new(ctx)));

    assert_eq!(stdout, String::from("Hello, world!\n")); // Echo adds newline.
    assert_eq!(stderr, String::new());
}

#[test]
fn execute_program_stderr() {
    let ctx = Context::default();
    let program = Program {
        statements: vec![Statement::AndOr(AndOr {
            operators: Vec::new(),
            pipelines: vec![Pipeline {
                is_async: false,
                segments: vec![PipelineSegment {
                    command: Command {
                        program: Word::Literal("echo".into()),
                        arguments: vec![Word::Literal("Hello, world!".into())],
                        redirects: vec![Redirect {
                            source: FileDescriptor::Number(1), // Stdout
                            target: FileDescriptor::Number(2), // Stderr
                            operator: pjsh_ast::RedirectOperator::Write,
                        }],
                    },
                }],
            }],
        })],
    };

    let (stdout, stderr) = crate::executor::execute_program(program, Arc::new(Mutex::new(ctx)));

    assert_eq!(stdout, String::new()); // Stdout is redirected to stderr.
    assert_eq!(stderr, String::from("Hello, world!\n")); // Echo adds newline.
}