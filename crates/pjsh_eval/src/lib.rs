mod alias;
mod command;
mod interpolate;
mod redirect;

use std::sync::Arc;

use command::execute_command;
use interpolate::interpolate_word;
use parking_lot::Mutex;
use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Command, Pipeline, PipelineSegment, Program, Redirect, Statement,
};
use pjsh_core::{Context, Scope};

use crate::{alias::replace_aliases, interpolate::interpolate};

type EvalResult<T> = Result<T, EvalError>;

type Status = i32;
const SUCCESS: Status = 0;

#[derive(Debug)]
pub enum EvalError {
    InvalidRedirect(Redirect),
    Message(String),
    UndefinedVariable(String),
    UnknownFileDescriptor(String),
}

pub fn eval_program(program: &Program, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    for statement in &program.statements {
        eval_statement(statement, Arc::clone(&ctx))?;
    }

    Ok(())
}

fn eval_statement(statement: &Statement, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    match statement {
        Statement::AndOr(and_or) => eval_and_or(and_or, ctx),
        Statement::Assignment(assignment) => eval_assignment(assignment, ctx),
        Statement::Function(_) => todo!(),
        Statement::If(_) => todo!(),
        Statement::While(_) => todo!(),
        Statement::Subshell(program) => eval_subshell(program, ctx),
    }
}

fn eval_and_or(and_or: &AndOr, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    let mut operators = and_or.operators.iter().peekable();
    let mut pipelines = and_or.pipelines.iter().peekable();

    loop {
        if pipelines.peek().is_none() {
            break;
        }

        let pipeline = pipelines.next().unwrap();
        eval_pipeline(pipeline, Arc::clone(&ctx))?;
        let is_ok = ctx.lock().last_exit == SUCCESS;

        match operators.next() {
            Some(AndOrOp::And) if !is_ok => break,
            Some(AndOrOp::Or) if is_ok => break,
            None => assert!(pipelines.peek().is_none()),
            _ => (),
        }
    }

    Ok(())
}

fn eval_pipeline(pipeline: &Pipeline, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    if let [segment] = &pipeline.segments[..] {
        return eval_pipeline_segment(segment, ctx);
    }

    todo!("eval multiple pipeline segments");
}

fn eval_pipeline_segment(segment: &PipelineSegment, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    match segment {
        PipelineSegment::Command(command) => eval_command(command, ctx),
        PipelineSegment::Condition(_condition) => todo!(),
    }
}

fn eval_assignment(assignment: &Assignment, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    let variable_name = interpolate_word(&assignment.key, Arc::clone(&ctx))?;
    let variable_value = interpolate_word(&assignment.value, Arc::clone(&ctx))?;
    ctx.lock().set_var(variable_name, variable_value);
    Ok(())
}

fn eval_subshell(subshell: &Program, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    let scope_name = format!("{} subshell", ctx.lock().name());
    let interactive = ctx.lock().is_interactive();
    ctx.lock()
        .push_scope(Scope::new_named(scope_name, interactive));

    let result = eval_program(subshell, Arc::clone(&ctx));

    ctx.lock().pop_scope();
    result
}

pub fn eval_command(command: &Command, ctx: Arc<Mutex<Context>>) -> EvalResult<()> {
    let mut args = interpolate(&command.arguments, Arc::clone(&ctx))?;
    replace_aliases(&mut args, Arc::clone(&ctx));

    args.make_contiguous();
    let args = args.as_slices().0;

    execute_command(args, &command.redirects[..], ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pjsh_ast::Word;

    #[test]
    fn it_works() {
        let program = Program {
            statements: vec![Statement::AndOr(AndOr {
                operators: vec![],
                pipelines: vec![Pipeline {
                    is_async: false,
                    segments: vec![PipelineSegment::Command(Command {
                        arguments: vec![
                            Word::Literal("echo".to_owned()),
                            Word::Quoted("Hello, world!".to_owned()),
                        ],
                        redirects: vec![Redirect {
                            source: pjsh_ast::FileDescriptor::Number(1),
                            target: pjsh_ast::FileDescriptor::Number(2),
                            operator: pjsh_ast::RedirectOperator::Write,
                        }],
                    })],
                }],
            })],
        };
        let ctx = Arc::new(Mutex::new(Context::default()));
        eval_program(&program, ctx).unwrap();
    }
}
