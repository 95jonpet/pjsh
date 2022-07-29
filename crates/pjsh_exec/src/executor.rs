use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    io::{BufRead, BufReader, Read, Seek},
    path::{Path, PathBuf},
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
    Condition, Context, Scope,
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

        ctx.lock().register_exit(exit_status);
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
            // (the condition exits with a non 0 code).
            if ctx.lock().last_exit() != EXIT_SUCCESS {
                continue;
            }

            ctx.lock().register_exit(0);
            return self.execute_statements(branch.statements, Arc::clone(&ctx), fds);
        }

        ctx.lock().register_exit(0); // Ensure that conditionals don't taint the scope.

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
            // exits with a non 0 code).
            self.execute_and_or(conditional.condition.clone(), Arc::clone(&ctx), fds);
            if ctx.lock().last_exit() != EXIT_SUCCESS {
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
                    let expanded = expand(input, Arc::clone(&ctx), self);
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
            Statement::Function(function) => ctx.lock().register_function(function),
            Statement::Subshell(subshell) => {
                ctx.lock().push_scope(Scope::new(
                    "subshell".to_owned(),
                    None,
                    Some(HashMap::default()),
                    Some(HashMap::default()),
                    HashSet::default(),
                    false,
                ));

                for subshell_statement in subshell.statements {
                    self.execute_statement(subshell_statement, ctx.clone(), fds);
                }

                ctx.lock().pop_scope();
            }
            Statement::If(conditionals) => self.execute_conditional_chain(conditionals, ctx, fds),
            Statement::While(conditional) => self.execute_conditional_loop(conditional, ctx, fds),
        }
    }

    /// Executes an [`Assignment`] by modifying the [`Context`].
    fn execute_assignment(&self, assignment: Assignment, context: Arc<Mutex<Context>>) {
        let key = interpolate_word(self, assignment.key, Arc::clone(&context));
        let value = interpolate_word(self, assignment.value, Arc::clone(&context));
        context.lock().set_var(key, value);
    }

    /// Executes a [`Command`].
    fn execute_command(
        &self,
        command: Command,
        context: Arc<Mutex<Context>>,
        fds: &mut FileDescriptors,
    ) -> Result<Value, ExecError> {
        redirect_file_descriptors(fds, Arc::clone(&context), self, &command.redirects)?;

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
        let maybe_function = context.lock().get_function(&program).cloned();
        if let Some(function) = maybe_function {
            let mut function_vars = HashMap::with_capacity(function.args.len());
            for arg_name in function.args {
                match args.pop_front() {
                    Some(arg_value) => function_vars.insert(arg_name, arg_value),
                    None => return Err(ExecError::MissingFunctionArgument(arg_name)),
                };
            }

            // Ensure that $0 is set to the program's name.
            args.push_front(function.name.clone());

            let function_scope = Scope::new(
                function.name,
                Some(Vec::from(args)),
                Some(function_vars),
                Some(HashMap::new()),
                HashSet::new(),
                false,
            );

            context.lock().push_scope(function_scope);

            let executor = self.clone();
            let fds = fds.try_clone().expect("clone file descriptor");
            let thread_handle = thread::spawn(move || {
                executor.execute_statements(function.body.statements, Arc::clone(&context), &fds);
                context.lock().pop_scope();
                context.lock().last_exit()
            });

            return Ok(Value::Thread(thread_handle));
        }

        // Attempt to start a program from an absolute path, or from a path relative to
        // $PWD if the program looks like a path.
        let ctx = &context.lock();
        if program.starts_with('.') || program.contains('/') {
            let program_path = resolve_path(ctx, &program);
            if !program_path.is_file() {
                return Err(ExecError::InvalidProgramPath(program_path));
            }

            // Handle any shebang if present.
            if let Some(shebang) = read_shebang(&program_path) {
                let mut shebang_args = shebang.split_whitespace();
                let shebang_program = shebang_args.next().expect("shebang program").to_owned();

                // Arrange arguments such that the following order holds:
                // shebang_program [shebang_args..] program [args..]
                args.push_front(path_to_string(program_path));
                for arg in shebang_args.rev() {
                    args.push_front(arg.to_owned());
                }

                return self.start_program(PathBuf::from(shebang_program), args, fds, ctx);
            }

            return self.start_program(program_path, args, fds, ctx);
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
            context: ctx,
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
        let cmd_name = cmd.name().to_owned();
        debug_assert_eq!(args.front(), Some(&cmd_name));

        // Create a new modified context for the command.
        ctx.lock().push_scope(Scope::new(
            cmd_name,
            Some(Vec::from(args)),
            None,
            None,
            HashSet::default(),
            false,
        ));

        let args = pjsh_core::command::Args {
            context: Arc::clone(&ctx),
            io: fds.io(),
        };

        let result = cmd.run(args);

        // Perform all asynchronous actions that have been requested.
        let mut code = result.code;
        for action in result.actions {
            if let Some(action_code) = self.perform_action(action, Arc::clone(&ctx), &mut fds) {
                if action_code != 0 {
                    code = action_code;
                }
            }
        }

        ctx.lock().pop_scope();

        code
    }

    /// Performs an asynchronous action that requested by a command.
    fn perform_action(
        &self,
        action: Action,
        context: Arc<Mutex<Context>>,
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
                self.resolve_command_type(&name, &context.lock()),
            )),
            Action::ResolveCommandPath(name, callback) => Some(callback(
                name.clone(),
                fds.io(),
                find_in_path(&name, &context.lock()).as_ref(),
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
        ctx: Arc<Mutex<Context>>,
        args: Vec<String>,
        script_file: PathBuf,
        fds: &mut FileDescriptors,
    ) -> Option<i32> {
        let old_args = ctx.lock().replace_args(args);
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
        ctx.lock().replace_args(old_args);
        Some(ctx.lock().last_exit())
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
        cmd.envs(context.exported_vars());
        cmd.args(args);

        if let Some(path) = context.get_var("PWD") {
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

        if let Some(alias) = ctx.aliases.get(name) {
            return CommandType::Alias(alias.to_owned());
        }

        if ctx.get_function(name).is_some() {
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
        expand(command.arguments, context, self)
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
    ctx: Arc<Mutex<Context>>,
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
                if let Some(file_path) = expand_single(file_path, Arc::clone(&ctx), executor) {
                    let path = resolve_path(&ctx.lock(), &file_path);
                    match redirect.mode {
                        pjsh_ast::RedirectMode::Write => {
                            fds.set(source, FileDescriptor::File(path));
                        }
                        pjsh_ast::RedirectMode::Append => {
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
                if let Some(file_path) = expand_single(file_path, Arc::clone(&ctx), executor) {
                    let path = resolve_path(&ctx.lock(), &file_path);
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

/// Reads the shebang from the first line of a file.
fn read_shebang<P: AsRef<Path>>(path: P) -> Option<String> {
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mut buffer = BufReader::new(file);
    let mut first_line = String::new();
    if buffer.read_line(&mut first_line).is_err() {
        return None;
    }

    if first_line.starts_with("#!") {
        return Some(first_line.trim_start_matches("#!").to_owned());
    }

    None
}
