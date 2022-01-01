use std::{
    collections::{HashMap, VecDeque},
    io::{BufReader, Read, Seek},
    path::PathBuf,
    process,
    sync::Arc,
    thread,
};

use parking_lot::Mutex;
use pjsh_ast::{AndOr, AndOrOp, Assignment, Command, Pipeline, Statement};
use pjsh_core::{
    find_in_path,
    utils::{path_to_string, resolve_path},
    Context, InternalCommand, InternalIo,
};
use tempfile::tempfile;

use crate::{
    builtins::builtin,
    error::ExecError,
    exit::{EXIT_GENERAL_ERROR, EXIT_SUCCESS},
    expand::{expand, expand_single},
    io::{FileDescriptor, FileDescriptors, FD_STDERR, FD_STDIN, FD_STDOUT},
    word::interpolate_word,
};

#[derive(Debug)]
enum Value {
    Process(process::Child),
    Thread(thread::JoinHandle<i32>),
}

/// An executor is responsible for executing a parsed AST.
#[derive(Default)]
pub struct Executor {
    actions: HashMap<String, Box<dyn InternalCommand>>,
}

impl Executor {
    /// Creates an empty executor.
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    /// Registers a built-in [`InternalCommand`] action within the executor.
    ///
    /// Any previous built-in action with the same name is replaced.
    pub fn register_action(&mut self, builtin: Box<dyn InternalCommand>) {
        self.actions.insert(builtin.name().to_string(), builtin);
    }

    /// Executes an [`AndOr`].
    pub fn execute_and_or(&self, and_or: AndOr, ctx: Arc<Mutex<Context>>, fds: &FileDescriptors) {
        debug_assert!(and_or.operators.len() == and_or.pipelines.len() - 1);
        let mut operators = and_or.operators.iter();
        let mut exit_status = EXIT_SUCCESS;
        let mut operator = &AndOrOp::And;

        for pipeline in and_or.pipelines {
            let is_accepting_segment = match operator {
                AndOrOp::And => exit_status == EXIT_SUCCESS,
                AndOrOp::Or => exit_status != EXIT_SUCCESS,
            };

            if !is_accepting_segment {
                break;
            }

            exit_status = self.execute_pipeline(pipeline, Arc::clone(&ctx), fds);
            operator = operators.next().unwrap_or(&AndOrOp::And); // There are n-1 operators.
        }

        ctx.lock().last_exit = exit_status;
    }

    /// Executes a [`Pipeline`].
    pub fn execute_pipeline(
        &self,
        pipeline: Pipeline,
        ctx: Arc<Mutex<Context>>,
        fds: &FileDescriptors,
    ) -> i32 {
        let mut values = VecDeque::with_capacity(pipeline.segments.len());
        let mut segments = pipeline.segments.into_iter().peekable();
        let mut fds = fds.try_clone().unwrap(); // Clone to allow local modification.
        let mut stdout = fds.take(&FD_STDOUT); // The original stdout.
        loop {
            let segment = segments.next().unwrap();
            let is_last = segments.peek().is_none();

            // Create a new pipe for all but the last pipeline segment.
            if is_last {
                fds.set(FD_STDOUT, stdout.take().unwrap_or(FileDescriptor::Stdout));
            } else {
                fds.set(FD_STDOUT, FileDescriptor::Pipe(os_pipe::pipe().unwrap()));
            }

            let result = self.execute_command(segment.command, Arc::clone(&ctx), &mut fds);
            match result {
                Ok(value) => {
                    // Set the pipe that was previously used for output as input for the next
                    // pipeline segment. The output end is replaced in the next loop iteration.
                    // This also serves to release stdout from the previous process.
                    if let Some(previous_stdout) = fds.take(&FD_STDOUT) {
                        fds.set(FD_STDIN, previous_stdout);
                    }

                    values.push_back(value);
                }
                Err(err) => {
                    ctx.lock().host.lock().eprintln(&format!("pjsh: {}", &err));
                }
            }

            if is_last {
                break;
            }
        }

        // Register async processes and threads for later processing.
        if pipeline.is_async {
            while let Some(value) = values.pop_front() {
                match value {
                    Value::Process(process) => ctx.lock().host.lock().add_child_process(process),
                    Value::Thread(thread) => ctx.lock().host.lock().add_thread(thread),
                }
            }

            return EXIT_SUCCESS;
        }

        // Wait for the last synchronous process or thread to exit. Exit with 0 only if all pipeline
        // segments exit with 0. Iterate backwards to ensure clean termination of the final segment.
        let mut status = EXIT_SUCCESS;
        while let Some(value) = values.pop_back() {
            let segment_status = match value {
                Value::Process(mut child) => match child.wait() {
                    Ok(status) => status.code().unwrap_or(EXIT_GENERAL_ERROR),
                    Err(error) => {
                        ctx.lock()
                            .host
                            .lock()
                            .eprintln(&format!("failed to wait for process: {}", error));
                        EXIT_GENERAL_ERROR
                    }
                },
                Value::Thread(thread_handle) => thread_handle.join().unwrap_or(EXIT_GENERAL_ERROR),
            };

            if segment_status != EXIT_SUCCESS {
                status = segment_status;
            }
        }

        status
    }

    /// Executes a [`Statement`].
    pub fn execute_statement(
        &self,
        statement: Statement,
        context: Arc<Mutex<Context>>,
        fds: &FileDescriptors,
    ) {
        match statement {
            Statement::AndOr(and_or) => self.execute_and_or(and_or, context, fds),
            Statement::Assignment(assignment) => self.execute_assignment(assignment, context),
            Statement::Subshell(subshell) => {
                let inner_context = Arc::new(Mutex::new(context.lock().fork()));
                for subshell_statement in subshell.statements {
                    self.execute_statement(subshell_statement, Arc::clone(&inner_context), fds);
                }
            }
        }
    }

    /// Executes an [`Assignment`] by modifying the [`Context`].
    fn execute_assignment(&self, assignment: Assignment, context: Arc<Mutex<Context>>) {
        let key = interpolate_word(self, assignment.key, &context.lock());
        let value = interpolate_word(self, assignment.value, &context.lock());
        context.lock().scope.set_env(key, value);
    }

    /// Executes a [`Command`].
    fn execute_command(
        &self,
        command: Command,
        context: Arc<Mutex<Context>>,
        fds: &mut FileDescriptors,
    ) -> Result<Value, ExecError> {
        redirect_file_descriptors(fds, &context.lock(), self, &command.redirects)?;

        let mut args = self.expand(command, Arc::clone(&context));
        let program = args.pop_front().expect("program must be defined");

        if let Some(action) = self.actions.get(&program) {
            return execute_internal_command(action.clone(), args, context, fds);
        }

        // Attempt to use the program as a builtin.
        if let Some(builtin) = builtin(&program) {
            return execute_internal_command(builtin, args, context, fds);
        }

        // Attempt to start a program from an absolute path, or from a path relative to $PWD if the
        // program looks like a path.
        let ctx = &context.lock();
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

        Err(ExecError::UnknownProgram(program))
    }

    /// Starts a program as a [`std::process::Command`] by spawning a child process.
    /// This function does not wait for the command to complete.
    fn start_program(
        &self,
        program: PathBuf,
        args: VecDeque<String>,
        fds: &mut FileDescriptors,
        context: &Context,
    ) -> Result<Value, ExecError> {
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
            Ok(child) => Ok(Value::Process(child)),
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
    fn expand(&self, command: Command, context: Arc<Mutex<Context>>) -> VecDeque<String> {
        let mut args = Vec::with_capacity(command.arguments.len() + 1);
        args.push(command.program);
        for arg in command.arguments {
            args.push(arg);
        }

        expand(args, &context.lock(), self)
    }
}

/// Executes an internal command.
fn execute_internal_command(
    command: Box<dyn InternalCommand>,
    mut args: VecDeque<String>,
    ctx: Arc<Mutex<Context>>,
    fds: &mut FileDescriptors,
) -> Result<Value, ExecError> {
    args.make_contiguous();
    let io = Arc::new(Mutex::new(InternalIo::new(
        fds.reader(&FD_STDIN).unwrap().unwrap(),
        fds.writer(&FD_STDOUT).unwrap().unwrap(),
        fds.writer(&FD_STDERR).unwrap().unwrap(),
    )));
    let thread_handle =
        thread::spawn(move || command.run(args.as_slices().0, Arc::clone(&ctx), io));
    Ok(Value::Thread(thread_handle))
}

/// Executes a program and returns a tuple of stdout and stderr.
pub(crate) fn execute_program(
    executor: &Executor,
    program: pjsh_ast::Program,
    context: Arc<Mutex<Context>>,
) -> (String, String) {
    let stdout = tempfile().expect("create temporary file");
    let stderr = tempfile().expect("create temporary file");
    let mut fds = FileDescriptors::new();
    fds.set(
        FD_STDOUT,
        FileDescriptor::FileHandle(stdout.try_clone().unwrap()),
    );
    fds.set(
        FD_STDERR,
        FileDescriptor::FileHandle(stderr.try_clone().unwrap()),
    );

    for statement in program.statements {
        executor.execute_statement(statement, Arc::clone(&context), &fds);
    }

    let read_file = |mut file: std::fs::File| {
        let _ = file.seek(std::io::SeekFrom::Start(0));
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        let _ = buf_reader.read_to_string(&mut contents);
        contents
    };

    (read_file(stdout), read_file(stderr))
}

/// Handles [`FileDescriptor`] redirections for some [`FileDescriptors`] in a [`Context`].
fn redirect_file_descriptors(
    fds: &mut FileDescriptors,
    ctx: &Context,
    executor: &Executor,
    redirects: &[pjsh_ast::Redirect],
) -> std::result::Result<(), ExecError> {
    for redirect in redirects {
        match (redirect.source.clone(), redirect.target.clone()) {
            (pjsh_ast::FileDescriptor::Number(from), pjsh_ast::FileDescriptor::Number(to)) => {
                if let Some(fd) = fds.get(&to) {
                    let fd = fd.try_clone().expect("clone file descriptor");
                    fds.set(from, fd);
                } else {
                    return Err(ExecError::UnknownFileDescriptor(to.to_string()));
                }
            }
            (
                pjsh_ast::FileDescriptor::Number(source),
                pjsh_ast::FileDescriptor::File(file_path),
            ) => {
                if let Some(file_path) = expand_single(file_path, ctx, executor) {
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
                if let Some(file_path) = expand_single(file_path, ctx, executor) {
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
