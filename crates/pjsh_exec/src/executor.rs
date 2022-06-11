use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{BufRead, BufReader, Read, Seek},
    path::PathBuf,
    process,
    sync::Arc,
    thread,
};

use parking_lot::Mutex;
use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Command, ConditionalChain, ConditionalLoop, Pipeline,
    PipelineSegment, Statement,
};
use pjsh_core::{
    command::{self, Action, CommandType},
    find_in_path,
    utils::{path_to_string, resolve_path},
    Condition, Context,
};
use tempfile::tempfile;

use crate::{
    condition::parse_condition,
    error::ExecError,
    exit::{EXIT_GENERAL_ERROR, EXIT_SUCCESS},
    expand::{expand, expand_single},
    io::{FileDescriptor, FileDescriptors, FD_STDERR, FD_STDIN, FD_STDOUT},
    word::interpolate_word,
};

#[derive(Debug)]
enum Value {
    Constant(i32),
    Process(process::Child),
    Thread(thread::JoinHandle<i32>),
}

/// An executor is responsible for executing a parsed AST.
#[derive(Clone)]
pub struct Executor {
    /// Built-in commands keyed by their name.
    builtins: HashMap<String, Box<dyn command::Command>>,
}

impl Executor {
    /// Creates an empty executor.
    pub fn new(commands: Vec<Box<dyn command::Command>>) -> Self {
        let mut builtins = HashMap::with_capacity(commands.len());
        for command in commands {
            builtins.insert(command.name().to_owned(), command);
        }

        Self { builtins }
    }

    /// Registers a built-in [`command::Command`] within the executor.
    ///
    /// Any previous built-in command with the same name is replaced.
    pub fn register_command(&mut self, builtin: Box<dyn command::Command>) {
        self.builtins.insert(builtin.name().to_string(), builtin);
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

    /// Executes a [`ConditionalChain`].
    pub fn execute_conditional_chain(
        &self,
        conditionals: ConditionalChain,
        ctx: Arc<Mutex<Context>>,
        fds: &FileDescriptors,
    ) {
        debug_assert!(
            conditionals.branches.len() == conditionals.conditions.len()
                || conditionals.branches.len() == conditionals.conditions.len() + 1
        );
        let mut branches = conditionals.branches.into_iter();

        for condition in conditionals.conditions.into_iter() {
            let branch = branches.next().expect("branch exists");
            self.execute_and_or(condition, Arc::clone(&ctx), fds);

            // Skip to the next condition in the chain if the current condition is not met
            // (the condition exits with a non 0 code). Furthermore, set the last exit to 0
            // in order to not pollute the shell's last exit code from evaluating
            // if-statement conditions. An intentional side effect of this is that
            // unevaluated branches always exit with code 0.
            if std::mem::replace(&mut ctx.lock().last_exit, EXIT_SUCCESS) != EXIT_SUCCESS {
                continue;
            }

            return self.execute_statements(branch.statements, Arc::clone(&ctx), fds);
        }

        // The "else" branch does not have a condition. It is always executed if no
        // other condition has been met.
        if let Some(branch) = branches.next() {
            self.execute_statements(branch.statements, Arc::clone(&ctx), fds);
        }
    }

    /// Executes a [`ConditionalLoop`].
    pub fn execute_conditional_loop(
        &self,
        conditional: ConditionalLoop,
        ctx: Arc<Mutex<Context>>,
        fds: &FileDescriptors,
    ) {
        loop {
            // Evaluate the condition and break the loop if it is not met (the condition
            // exits with a non 0 code). Furthermore, set the last exit to 0 in order to not
            // pollute the shell's last exit code from evaluating while-loop conditions.
            // An intentional side effect of this is a while-loop exits with a code of 0
            // when the loop exits normally.
            self.execute_and_or(conditional.condition.clone(), Arc::clone(&ctx), fds);
            if std::mem::replace(&mut ctx.lock().last_exit, EXIT_SUCCESS) != EXIT_SUCCESS {
                break;
            }

            self.execute_statements(conditional.body.statements.clone(), Arc::clone(&ctx), fds);
        }
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

            let result = match segment {
                PipelineSegment::Command(command) => {
                    self.execute_command(command, Arc::clone(&ctx), &mut fds)
                }
                PipelineSegment::Condition(input) => {
                    let expanded = expand(input, &ctx.lock(), self);
                    let input: Vec<_> = expanded.iter().map(String::as_str).collect();
                    let maybe_condition = parse_condition(&input, &ctx.lock());
                    match maybe_condition {
                        Ok(condition) => {
                            self.evaluate_condition(condition, Arc::clone(&ctx), &mut fds)
                        }
                        Err(err) => Err(err),
                    }
                }
            };
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
                    values.push_back(Value::Constant(EXIT_GENERAL_ERROR));
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
                    Value::Constant(_) => (), // Constants are always synchronous.
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
                Value::Constant(constant) => constant,
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

    /// Executes a [`Vec<Statement>`].
    pub fn execute_statements(
        &self,
        statements: Vec<Statement>,
        ctx: Arc<Mutex<Context>>,
        fds: &FileDescriptors,
    ) {
        for statement in statements {
            self.execute_statement(statement, Arc::clone(&ctx), fds);
        }
    }

    /// Executes a [`Statement`].
    pub fn execute_statement(
        &self,
        statement: Statement,
        ctx: Arc<Mutex<Context>>,
        fds: &FileDescriptors,
    ) {
        match statement {
            Statement::AndOr(and_or) => self.execute_and_or(and_or, ctx, fds),
            Statement::Assignment(assignment) => self.execute_assignment(assignment, ctx),
            Statement::Function(function) => ctx.lock().scope.add_function(function),
            Statement::Subshell(subshell) => {
                let name = ctx.lock().name.clone();
                let inner_context = Arc::new(Mutex::new(ctx.lock().fork(name)));
                for subshell_statement in subshell.statements {
                    self.execute_statement(subshell_statement, Arc::clone(&inner_context), fds);
                }
            }
            Statement::If(conditionals) => self.execute_conditional_chain(conditionals, ctx, fds),
            Statement::While(conditional) => self.execute_conditional_loop(conditional, ctx, fds),
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

        // Attempt to use the program as a built-in command.
        if let Some(builtin) = self.builtins.get(&program).cloned() {
            // The command trait requires the first argument should be the
            // program name. Fulfill that requirement by reinserting it.
            args.push_front(program);

            let thread_fds = fds.try_clone().unwrap();
            let executor = self.clone();
            let thread_handle = thread::spawn(move || {
                executor.execute_builtin_command(builtin, context, thread_fds, args)
            });
            return Ok(Value::Thread(thread_handle));
        }

        // Attempt to use the program as a function.
        let maybe_function = context.lock().scope.get_function(&program);
        if let Some(function) = maybe_function {
            let mut inner_context = context.lock().fork(function.name.clone());

            for arg_name in function.args {
                match args.pop_front() {
                    Some(arg_value) => inner_context.scope.set_env(arg_name, arg_value),
                    None => return Err(ExecError::MissingFunctionArgument(arg_name)),
                }
            }

            // Finalize the forked context and enable thread sharing.
            args.push_front(function.name); // Ensure that $0 is set to the program's name.
            inner_context.arguments = Vec::from(args);
            let inner_context = Arc::new(Mutex::new(inner_context));

            let executor = self.clone();
            let fds = fds.try_clone().expect("clone file descriptor");
            let thread_handle = thread::spawn(move || {
                executor.execute_statements(
                    function.body.statements,
                    Arc::clone(&inner_context),
                    &fds,
                );
                inner_context.lock().last_exit
            });
            return Ok(Value::Thread(thread_handle));
        }

        // Attempt to start a program from an absolute path, or from a path relative to
        // $PWD if the program looks like a path.
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

    /// Evaluates a [`Condition`].
    fn evaluate_condition(
        &self,
        condition: Box<dyn Condition>,
        ctx: Arc<Mutex<Context>>,
        fds: &mut FileDescriptors,
    ) -> Result<Value, ExecError> {
        let args = pjsh_core::command::Args {
            context: ctx.lock().clone(),
            io: fds.io(),
        };

        Ok(match condition.evaluate(args) {
            true => Value::Constant(EXIT_SUCCESS),
            false => Value::Constant(EXIT_GENERAL_ERROR),
        })
    }

    /// Executes a built-in command within a context.
    ///
    /// The first argument of `args` is expected to be the command's name as
    /// returned by [`pjsh_core::command::Command::name()`].
    ///
    /// Returns an exit code.
    pub fn execute_builtin_command(
        &self,
        cmd: Box<dyn pjsh_core::command::Command>,
        ctx: Arc<Mutex<Context>>,
        mut fds: FileDescriptors,
        args: VecDeque<String>,
    ) -> i32 {
        // Ensure that the first argument ($0) is the command's name.
        debug_assert_eq!(args.front(), Some(&cmd.name().to_owned()));

        // Create a new modified context for the command.
        let mut inner_context = ctx.lock().clone();
        inner_context.arguments = Vec::from(args);
        let args = pjsh_core::command::Args {
            context: inner_context,
            io: fds.io(),
        };

        let result = cmd.run(args);

        // Perform all asynchronous that have been requested.
        let mut code = result.code;
        for action in result.actions {
            if let Some(action_code) = self.perform_action(action, &mut ctx.lock(), &mut fds) {
                if action_code != 0 {
                    code = action_code;
                }
            }
        }

        code
    }

    /// Performs an asynchronous action that requested by a command.
    fn perform_action(
        &self,
        action: Action,
        context: &mut Context,
        fds: &mut FileDescriptors,
    ) -> Option<i32> {
        match action {
            Action::Interpolate(text, callback) => match pjsh_parse::parse_interpolation(&text) {
                Ok(word) => {
                    let value = interpolate_word(self, word, context);
                    Some(callback(fds.io(), Ok(&value)))
                }
                Err(error) => Some(callback(fds.io(), Err(&error.to_string()))),
            },
            Action::ResolveCommandType(name, callback) => Some(callback(
                fds.io(),
                self.resolve_command_type(&name, context),
            )),
            Action::ResolveCommandPath(name, callback) => Some(callback(
                name.clone(),
                fds.io(),
                find_in_path(&name, context).as_ref(),
            )),
            Action::SourceFile(script_file, ctx, args) => {
                self.source_file(ctx, args, script_file, fds)
            }
        }
    }

    /// Sources a file within the current executor and context.
    ///
    /// Returns the exit code for the last command in the sourced file.
    fn source_file(
        &self,
        mut ctx: Context,
        args: Vec<String>,
        script_file: PathBuf,
        fds: &mut FileDescriptors,
    ) -> Option<i32> {
        ctx.arguments = args;
        let ctx = Arc::new(Mutex::new(ctx));
        let mut reader = BufReader::new(fs::File::open(script_file).unwrap());
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                _ => (),
            }

            match pjsh_parse::parse(&line) {
                Ok(program) => {
                    line = String::new();
                    self.execute_statements(program.statements, Arc::clone(&ctx), fds);
                }
                Err(pjsh_parse::ParseError::IncompleteSequence) => continue,
                _ => break,
            }
        }
        let ctx = &ctx.lock();
        Some(ctx.last_exit)
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

        if let Some(path) = context.scope.get_env("PWD") {
            cmd.current_dir(path);
        }

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

    /// Resolves the type of a command given a name.
    fn resolve_command_type(&self, name: &str, ctx: &Context) -> CommandType {
        if self.builtins.contains_key(name) {
            return CommandType::Builtin;
        }

        if let Some(alias) = ctx.scope.get_alias(name) {
            return CommandType::Alias(alias);
        }

        if ctx.scope.get_function(name).is_some() {
            return CommandType::Function;
        }

        if let Some(path) = find_in_path(name, ctx) {
            return CommandType::Program(path);
        }

        CommandType::Unknown
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

    executor.execute_statements(program.statements, Arc::clone(&context), &fds);

    let read_file = |mut file: std::fs::File| {
        let _ = file.seek(std::io::SeekFrom::Start(0));
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        let _ = buf_reader.read_to_string(&mut contents);

        // Trim any final newline that are normally used to separate the shell output and prompt.
        if let Some('\n') = contents.chars().last() {
            contents.truncate(contents.len() - 1);
            if let Some('\r') = contents.chars().last() {
                contents.truncate(contents.len() - 1);
            }
        }

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
