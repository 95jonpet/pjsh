use std::collections::{HashMap, HashSet};

use pjsh_ast::{AndOr, Assignment, Command, Pipeline, PipelineSegment, Statement, Word};
use pjsh_core::{Context, Scope};
use pjsh_eval::{execute_statement, EvalResult};

#[derive(Clone)]
struct TrueCommand;
impl pjsh_core::command::Command for TrueCommand {
    fn name(&self) -> &str {
        "true"
    }

    fn run<'a>(&self, _: &'a mut pjsh_core::command::Args) -> pjsh_core::command::CommandResult {
        pjsh_core::command::CommandResult::code(0)
    }
}

#[test]
fn it_assigns_variables() {
    let mut context = Context::with_scopes(vec![Scope::new(
        "scope".into(),
        Some(Vec::default()),
        HashMap::default(),
        HashMap::default(),
        HashSet::default(),
    )]);

    let statement = Statement::Assignment(Assignment {
        key: Word::Literal("key".into()),
        value: Word::Literal("value".into()),
    });

    assert!(execute_statement(&statement, &mut context).is_ok());
    assert_eq!(context.get_var("key"), Some("value"));
}

#[test]
fn it_works() -> EvalResult<()> {
    let mut context = Context::with_scopes(vec![Scope::new(
        "scope".into(),
        Some(Vec::default()),
        HashMap::default(),
        HashMap::default(),
        HashSet::default(),
    )]);
    context
        .builtins
        .insert("true".into(), Box::new(TrueCommand));

    let statement = Statement::AndOr(AndOr {
        operators: Vec::default(),
        pipelines: vec![Pipeline {
            is_async: false,
            segments: vec![PipelineSegment::Command(Command {
                arguments: vec![Word::Literal("true".into())],
                redirects: Vec::default(),
            })],
        }],
    });

    execute_statement(&statement, &mut context)?;
    assert_eq!(context.last_exit(), 0);
    Ok(())
}
