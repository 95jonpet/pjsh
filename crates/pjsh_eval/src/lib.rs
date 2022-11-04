use call::{call_builtin_command, call_external_program, call_function};
use error::{EvalError, EvalResult};
use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Command, ConditionalChain, ConditionalLoop, Pipeline, Program,
    Redirect, Statement,
};
use pjsh_core::{
    command::CommandResult, find_in_path, utils::resolve_path, Context, FileDescriptor,
};
use words::expand_words;
pub use words::interpolate_word;

mod call;
mod error;
mod words;

/// Executes a [`Vec<Statement>`].
fn execute_statements(statements: &[Statement], context: &mut Context) -> EvalResult<()> {
    for statement in statements {
        execute_statement(statement, context)?;
    }
    Ok(())
}

/// Executes a statement within a context.
pub fn execute_statement(statement: &Statement, context: &mut Context) -> EvalResult<()> {
    match statement {
        Statement::AndOr(and_or) => execute_and_or(and_or, context).map(|_| Ok(()))?,
        Statement::Assignment(assignment) => execute_assignment(assignment, context),
        Statement::Function(function) => {
            context.register_function(function.clone());
            Ok(())
        }
        Statement::If(conditionals) => execute_conditional_chain(conditionals, context),
        Statement::While(conditional) => execute_conditional_loop(conditional, context),
        Statement::Subshell(subshell) => {
            let inner_context = context.try_clone().map_err(EvalError::ContextCloneFailed)?;
            execute_subshell(subshell, inner_context)
        }
    }
}

/// Executes an assignment.
fn execute_assignment(assignment: &Assignment, context: &mut Context) -> EvalResult<()> {
    let key = interpolate_word(&assignment.key, context)?;
    let value = interpolate_word(&assignment.value, context)?;
    context.set_var(key, value);
    Ok(())
}

/// Executes a subshell program within its own context.
pub(crate) fn execute_subshell(subshell: &Program, mut context: Context) -> EvalResult<()> {
    execute_statements(&subshell.statements, &mut context)
}

/// Executes a conditional chain.
fn execute_conditional_chain(
    conditionals: &ConditionalChain,
    context: &mut Context,
) -> Result<(), EvalError> {
    assert!(
        conditionals.branches.len() == conditionals.conditions.len()
            || conditionals.branches.len() == conditionals.conditions.len() + 1
    );

    let mut branches = conditionals.branches.iter();

    for condition in conditionals.conditions.iter() {
        let branch = branches.next().expect("branch exists");
        execute_and_or(condition, context)?;

        // Skip to the next condition in the chain if the current condition is not met
        // (the condition exits with a non 0 code).
        if context.last_exit() != 0 {
            continue;
        }

        context.register_exit(0);
        return execute_statements(&branch.statements, context);
    }

    context.register_exit(0); // Ensure that conditionals don't taint the scope.

    // The "else" branch does not have a condition. It is always executed if no
    // other condition has been met.
    if let Some(branch) = branches.next() {
        return execute_statements(&branch.statements, context);
    }

    Ok(())
}

/// Executes a conditional loop.
fn execute_conditional_loop(
    conditional: &ConditionalLoop,
    context: &mut Context,
) -> Result<(), EvalError> {
    loop {
        // Evaluate the condition and break the loop if it is not met (the condition
        // exits with a non 0 code).
        if execute_and_or(&conditional.condition, context)? != 0 {
            break;
        }

        execute_statements(&conditional.body.statements, context)?;
    }
    Ok(())
}

/// Executes a sequence of and/or logic.
fn execute_and_or(and_or: &AndOr, context: &mut Context) -> EvalResult<i32> {
    assert_eq!(and_or.operators.len(), and_or.pipelines.len() - 1);
    let mut operators = and_or.operators.iter();
    let mut exit_status = 0;
    let mut operator = &AndOrOp::And;

    for pipeline in &and_or.pipelines {
        let is_accepting_segment = match operator {
            AndOrOp::And => exit_status == 0,
            AndOrOp::Or => exit_status != 0,
        };

        if !is_accepting_segment {
            break;
        }

        exit_status = execute_pipeline(pipeline, context)?;
        operator = operators.next().unwrap_or(&AndOrOp::And); // There are n-1 operators.
    }

    context.register_exit(exit_status);
    Ok(exit_status)
}

/// Executes a pipeline.
fn execute_pipeline(pipeline: &Pipeline, context: &mut Context) -> EvalResult<i32> {
    if pipeline.segments.is_empty() {
        return Ok(0); // Empty pipelines cannot fail.
    }

    // Prepare commands.
    let mut commands = Vec::with_capacity(pipeline.segments.len());
    for segment in &pipeline.segments {
        match segment {
            pjsh_ast::PipelineSegment::Command(command) => {
                commands.push(execute_command(command, context)?)
            }
            pjsh_ast::PipelineSegment::Condition(_) => todo!(),
        }
    }

    // Override stdin and stdout of all relevant segments.
    for i in 0..(pipeline.segments.len() - 1) {
        let (reader, writer) = os_pipe::pipe().map_err(EvalError::CreatePipeFailed)?;
        if let CommandResult::Process(process) = &mut commands[i] {
            process.command.stdout(writer);
        }
        if let CommandResult::Process(process) = &mut commands[i + 1] {
            process.command.stdin(reader);
        }
    }

    // Start the child processes.
    let mut exit_code = 0;
    let mut processes = Vec::with_capacity(commands.len());
    let mut io_errors = Vec::new();
    for command in commands {
        match command {
            CommandResult::Builtin(builtin) => exit_code = builtin.code,
            CommandResult::Process(mut process) => match process.command.spawn() {
                Ok(process) => processes.push(process),
                Err(error) => {
                    io_errors.push(error);
                    break;
                }
            },
        }
    }

    // Wait for synchronous processes to terminate.
    // Register asyncronous processes in the shell.
    // Register and return all pipeline errors.
    if pipeline.is_async && io_errors.is_empty() {
        let mut host = context.host.lock();
        for process in processes {
            host.add_child_process(process);
        }
        Ok(0)
    } else {
        for mut process in processes {
            match process.wait() {
                Ok(exit_status) => match exit_status.code() {
                    Some(code) => exit_code = code,
                    None => exit_code = 127,
                },
                Err(error) => io_errors.push(error),
            }
        }

        if !io_errors.is_empty() {
            return Err(EvalError::PipelineFailed(io_errors));
        }

        Ok(exit_code)
    }
}

/// Executes a command.
fn execute_command(command: &Command, context: &mut Context) -> EvalResult<CommandResult> {
    redirect_file_descriptors(&command.redirects, context)?;
    let args = expand_words(&command.arguments, context)?;

    if let Some(builtin) = context.get_builtin(&args[0]).cloned() {
        return call_builtin_command(&builtin, &args, context);
    }

    if let Some(function) = context.get_function(&args[0]).cloned() {
        call_function(&function, &args, context)?;
        return Ok(CommandResult::code(0));
    }

    if let Some(program) = find_in_path(&args[0], context) {
        // let cmd = call_external_program(&program, &args[1..], context)?;
        // return Ok(CommandResult::from()
        return call_external_program(&program, &args[1..], context).map(CommandResult::from);
    }

    Err(EvalError::UnknownCommand(args[0].to_owned()))
}

/// Redirects file descriptors.
fn redirect_file_descriptors(redirects: &[Redirect], context: &mut Context) -> EvalResult<()> {
    for redirect in redirects {
        redirect_file_descriptor(redirect, context)?;
    }
    Ok(())
}

/// Redirects a file descriptor.
fn redirect_file_descriptor(redirect: &Redirect, context: &mut Context) -> EvalResult<()> {
    match (&redirect.source, &redirect.target) {
        (pjsh_ast::FileDescriptor::Number(source), pjsh_ast::FileDescriptor::Number(target)) => {
            if let Some(file_descriptor) = context.get_file_descriptor(*target) {
                context.set_file_descriptor(*source, file_descriptor.try_clone().unwrap());
            } else {
                return Err(EvalError::UndefinedFileDescriptor(*target));
            }
        }
        (pjsh_ast::FileDescriptor::Number(source), pjsh_ast::FileDescriptor::File(file_path)) => {
            let path = resolve_path(context, interpolate_word(file_path, context)?);
            let file_descriptor = match redirect.mode {
                pjsh_ast::RedirectMode::Write => FileDescriptor::File(path),
                pjsh_ast::RedirectMode::Append => FileDescriptor::File(path),
            };
            context.set_file_descriptor(*source, file_descriptor);
        }
        (pjsh_ast::FileDescriptor::File(file_path), pjsh_ast::FileDescriptor::Number(target)) => {
            let path = resolve_path(context, interpolate_word(file_path, context)?);
            context.set_file_descriptor(*target, FileDescriptor::File(path));
        }
        (pjsh_ast::FileDescriptor::File(_), pjsh_ast::FileDescriptor::File(_)) => unreachable!(),
    };

    Ok(())
}
