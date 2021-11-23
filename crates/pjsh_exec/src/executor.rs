use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    io::Write,
    path::PathBuf,
    process::{self, Child, Stdio},
    rc::Rc,
};

use pjsh_ast::{
    AndOr, AndOrOp, Assignment, Command, FileDescriptor, Pipeline, RedirectOperator, Statement,
};
use pjsh_builtins::all_builtins;
use pjsh_core::{find_in_path, BuiltinCommand, Context, ExecError, Result, Value};

use crate::{expand, word::interpolate_word, Input};

pub struct Executor {
    builtins: HashMap<String, Box<dyn BuiltinCommand>>,
}

impl Executor {
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

    pub fn execute_pipeline(&self, pipeline: Pipeline, context: Rc<RefCell<Context>>) {
        let mut segments = pipeline.segments.into_iter().peekable();
        let mut last_child = None;
        let mut last_value = None;
        loop {
            let segment = segments.next().unwrap();
            let stdout = segments.peek().map(|_| Stdio::piped());
            let is_last = segments.peek().is_none();

            let stdin = std::mem::take(&mut last_child)
                .map(|c: Child| c.stdout)
                .flatten();
            let input = match stdin {
                Some(stdin) => Input::Piped(stdin),
                None => Input::Value(last_value.take().unwrap_or_else(String::new)),
            };
            let result = self.execute_command(segment.command, Rc::clone(&context), input, stdout);
            last_child = None;
            match result {
                Ok(value) => match value {
                    Value::Child(child) => {
                        last_child = Some(child);
                        last_value = None;
                    }
                    Value::String(string) => last_value = Some(string),
                    Value::Empty => last_value = Some(String::new()),
                },
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
        } else if let Some(value) = last_value {
            if !value.is_empty() {
                context.borrow().host.lock().println(&value);
            }
        }
    }

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

    fn execute_command(
        &self,
        command: Command,
        context: Rc<RefCell<Context>>,
        stdin: Input,
        stdout: Option<Stdio>,
    ) -> Result {
        // Allow stdout to be redirected to a file.
        // TODO: Refactor and generalize.
        let mut stdout = stdout;
        for redirect in &command.redirects {
            if redirect.source == FileDescriptor::Number(1)
                && redirect.operator == RedirectOperator::Write
            {
                if let FileDescriptor::File(file_name) = &redirect.target {
                    let mut path = context
                        .borrow()
                        .scope
                        .get_env("PWD")
                        .map(PathBuf::from)
                        .unwrap();
                    path.push(interpolate_word(file_name.clone(), &context.borrow()));
                    let file = std::fs::File::create(path).unwrap();
                    stdout = Some(Stdio::from(file));
                }
            }
        }

        let mut args = self.expand(command, Rc::clone(&context));
        let program = args.pop_front().expect("program must be defined");

        // Attempt to use the program as a builtin.
        if let Some(builtin) = self.builtins.get(&program) {
            args.make_contiguous();
            return builtin.run(args.as_slices().0, &mut context.borrow_mut());
        }

        // Search for the program in $PATH and spawn a child process for it if possible.
        let ctx = &context.borrow();
        if let Some(executable) = find_in_path(&program, ctx) {
            self.start_program(executable, args, stdin, stdout, ctx)
        } else {
            Err(ExecError::Message(format!("unknown program: {}", &program)))
        }
    }

    /// Starts a program as a [`std::process::Command`] by spawning a child process.
    /// This function does not wait for the command to complete.
    fn start_program(
        &self,
        program: PathBuf,
        args: VecDeque<String>,
        stdin: Input,
        stdout: Option<Stdio>,
        context: &Context,
    ) -> Result {
        let mut cmd = process::Command::new(program);
        cmd.envs(context.scope.envs());
        cmd.args(args);

        let mut value = None;
        if let Input::Piped(pipe) = stdin {
            cmd.stdin(pipe);
        } else if let Input::Value(string) = stdin {
            value = Some(string);
            cmd.stdin(Stdio::piped());
        }

        if let Some(stdout) = stdout {
            cmd.stdout(stdout);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                if let Some(string) = value {
                    let _ = write!(child.stdin.take().expect("stdin exists"), "{}", string);
                }

                Ok(Value::Child(child))
            }
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => unreachable!("Should be caught in caller"),
                _ => Err(ExecError::ChildSpawnFailed),
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

impl Default for Executor {
    fn default() -> Self {
        let mut builtins: HashMap<String, Box<dyn BuiltinCommand>> = HashMap::new();
        for builtin in all_builtins() {
            builtins.insert(builtin.name().to_string(), builtin);
        }

        Self { builtins }
    }
}
