use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    path::PathBuf,
    process,
    rc::Rc,
};

use pjsh_ast::{AndOr, AndOrOp, Assignment, Command, Pipeline, Statement};
use pjsh_builtins::all_builtins;
// use pjsh_builtins::all_builtins;
use pjsh_core::{
    find_in_path,
    utils::{path_to_string, resolve_path},
    Context, ExecError, InternalCommand, InternalIo, Result,
};

use crate::{
    expand::{expand, expand_single},
    io::{FileDescriptor, FileDescriptors, FD_STDERR, FD_STDIN, FD_STDOUT},
    word::interpolate_word,
};

/// An executor is responsible for executing a parsed AST.
pub struct Executor {
    /// Shell builtins that can be used as commands.
    builtins: HashMap<String, Box<dyn InternalCommand>>,
}

impl Executor {
    /// Executes an [`AndOr`].
    pub fn execute_and_or(&self, and_or: AndOr, context: Rc<RefCell<Context>>) {
        let mut operators = and_or.operators.iter();
        let mut exit_status = 0;
        let mut operator = &AndOrOp::And;

        for pipeline in and_or.pipelines {
            match (operator, exit_status) {
                (AndOrOp::And, 0) => self.execute_pipeline(pipeline, Rc::clone(&context)),
                (AndOrOp::Or, i) if i != 0 => self.execute_pipeline(pipeline, Rc::clone(&context)),
                _ => break,
            }
            operator = operators.next().unwrap_or(&AndOrOp::And);
            exit_status = context.borrow().last_exit;
        }
    }

    /// Executes a [`Pipeline`].
    pub fn execute_pipeline(&self, pipeline: Pipeline, context: Rc<RefCell<Context>>) {
        let mut segments = pipeline.segments.into_iter().peekable();
        let mut fds = FileDescriptors::new();
        let mut last_child = None;
        loop {
            let segment = segments.next().unwrap();
            let is_last = segments.peek().is_none();

            // Create a new pipe for all but the last pipeline segment.
            if is_last {
                fds.set(FD_STDOUT, FileDescriptor::Stdout);
            } else {
                fds.set(FD_STDOUT, FileDescriptor::Pipe(os_pipe::pipe().unwrap()));
            }

            let result = self.execute_command(segment.command, Rc::clone(&context), &mut fds);
            match result {
                Ok(Some(child)) => {
                    // Set the pipe that was previously used for output as input for the next
                    // pipeline segment. The output end is replaced in the next loop iteration.
                    if let Some(stdout) = fds.take(&FD_STDOUT) {
                        fds.set(FD_STDIN, stdout);
                    }

                    last_child = Some(child);
                }
                Ok(None) => {}
                Err(err) => {
                    context
                        .borrow()
                        .host
                        .lock()
                        .eprintln(&format!("pjsh: {}", &err));
                }
            }

            if is_last {
                break;
            }
        }

        if let Some(mut child) = last_child {
            if pipeline.is_async {
                context
                    .borrow_mut()
                    .host
                    .lock()
                    .println(&format!("pjsh: PID {} started", child.id()));
                context.borrow_mut().host.lock().add_child_process(child);
            } else {
                let _ = child.wait();
            }
        }
    }

    /// Executes a [`Statement`].
    pub fn execute_statement(&self, statement: Statement, context: Rc<RefCell<Context>>) {
        match statement {
            Statement::AndOr(and_or) => self.execute_and_or(and_or, context),
            Statement::Assignment(assignment) => self.execute_assignment(assignment, context),
        }
    }

    /// Executes an [`Assignment`] by modifying the [`Context`].
    fn execute_assignment(&self, assignment: Assignment, context: Rc<RefCell<Context>>) {
        let key = interpolate_word(assignment.key, &context.borrow());
        let value = interpolate_word(assignment.value, &context.borrow());
        context.borrow_mut().scope.set_env(key, value);
    }

    /// Executes a [`Command`].
    fn execute_command(
        &self,
        command: Command,
        context: Rc<RefCell<Context>>,
        fds: &mut FileDescriptors,
    ) -> Result {
        redirect_file_descriptors(fds, &context.borrow(), &command.redirects)?;

        let mut args = self.expand(command, Rc::clone(&context));
        let program = args.pop_front().expect("program must be defined");

        // Attempt to use the program as a builtin.
        if let Some(builtin) = self.builtins.get(&program) {
            args.make_contiguous();
            // TODO: Run on a separate thread to allow async pipelines.
            // TODO: Handle unwrapping.
            let mut io = InternalIo::new(
                fds.reader(&FD_STDIN).unwrap().unwrap(),
                fds.writer(&FD_STDOUT).unwrap().unwrap(),
                fds.writer(&FD_STDERR).unwrap().unwrap(),
            );
            builtin.run(args.as_slices().0, &mut context.borrow_mut(), &mut io);
            return Ok(None);
        }

        // Attempt to start a program from an absolute path, or from a path relative to $PWD if the
        // program looks like a path.
        let ctx = &context.borrow();
        if program.starts_with('.') || program.contains('/') {
            let program_in_pwd = resolve_path(ctx, &program);
            if program_in_pwd.is_file() {
                return self.start_program(program_in_pwd, args, fds, ctx);
            }
        }

        // Search for the program in $PATH and spawn a child process for it if possible.
        if let Some(executable) = find_in_path(&program, ctx) {
            return self.start_program(executable, args, fds, ctx);
        }

        Err(ExecError::Message(format!("unknown program: {}", &program)))
    }

    /// Starts a program as a [`std::process::Command`] by spawning a child process.
    /// This function does not wait for the command to complete.
    fn start_program(
        &self,
        program: PathBuf,
        args: VecDeque<String>,
        fds: &mut FileDescriptors,
        context: &Context,
    ) -> Result {
        let mut cmd = process::Command::new(program.clone());
        cmd.envs(context.scope.envs());
        cmd.args(args);

        if let Some(stdin) = fds.input(&FD_STDIN) {
            cmd.stdin(stdin?);
        }
        if let Some(stdout) = fds.output(&FD_STDOUT) {
            cmd.stdout(stdout?);
        }
        if let Some(stderr) = fds.output(&FD_STDERR) {
            cmd.stderr(stderr?);
        }

        match cmd.spawn() {
            Ok(child) => Ok(Some(child)),
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => unreachable!("Should be caught in caller"),
                _ => Err(ExecError::ChildSpawnFailed(
                    error.to_string().replace("%1", &path_to_string(&program)),
                )),
            },
        }
    }

    /// Expands a [`Command`] into a [`VecDeque`] of arguments.
    /// Evaluates variables and resolves aliases.
    fn expand(&self, command: Command, context: Rc<RefCell<Context>>) -> VecDeque<String> {
        let mut args = Vec::with_capacity(command.arguments.len() + 1);
        args.push(command.program);
        for arg in command.arguments {
            args.push(arg);
        }

        expand(args, &context.borrow())
    }
}

/// Handles [`FileDescriptor`] redirections for some [`FileDescriptors`] in a [`Context`].
fn redirect_file_descriptors(
    fds: &mut FileDescriptors,
    ctx: &Context,
    redirects: &[pjsh_ast::Redirect],
) -> std::result::Result<(), ExecError> {
    for redirect in redirects {
        match (redirect.source.clone(), redirect.target.clone()) {
            (pjsh_ast::FileDescriptor::Number(_), pjsh_ast::FileDescriptor::Number(_)) => {
                todo!("general file descriptor redirection");
            }
            (
                pjsh_ast::FileDescriptor::Number(source),
                pjsh_ast::FileDescriptor::File(file_path),
            ) => {
                if let Some(file_path) = expand_single(file_path, ctx) {
                    let path = resolve_path(ctx, &file_path);
                    match redirect.operator {
                        pjsh_ast::RedirectOperator::Write => {
                            fds.set(source, FileDescriptor::File(path));
                        }
                        pjsh_ast::RedirectOperator::Append => {
                            fds.set(source, FileDescriptor::AppendFile(path));
                        }
                    }
                } else {
                    // TODO: Print error with the fully expanded file name word.
                    return Err(ExecError::Message(format!(
                        "invalid redirect: {:?}",
                        redirect
                    )));
                }
            }
            (
                pjsh_ast::FileDescriptor::File(file_path),
                pjsh_ast::FileDescriptor::Number(target),
            ) => {
                if let Some(file_path) = expand_single(file_path, ctx) {
                    let path = resolve_path(ctx, &file_path);
                    fds.set(target, FileDescriptor::File(path));
                } else {
                    // TODO: Print error with the fully expanded file name word.
                    return Err(ExecError::Message(format!(
                        "invalid redirect: {:?}",
                        redirect
                    )));
                }
            }
            (pjsh_ast::FileDescriptor::File(_), pjsh_ast::FileDescriptor::File(_)) => {
                unreachable!("cannot redirect input from file to file");
            }
        }
    }

    Ok(())
}

impl Default for Executor {
    fn default() -> Self {
        let mut builtins: HashMap<String, Box<dyn InternalCommand>> = HashMap::new();
        for builtin in all_builtins() {
            builtins.insert(builtin.name().to_string(), builtin);
        }

        Self { builtins }
    }
}
