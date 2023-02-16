mod builtins;

mod error;
mod shell;

use std::fs::{read_to_string, File};
use std::process::ExitCode;
use std::{env::current_exe, path::PathBuf, sync::Arc};

use clap::{crate_version, Parser};
use error::{ErrorHandler, GuidingErrorHandler, SimpleErrorHandler};
use parking_lot::Mutex;
use pjsh_complete::Completer;
use pjsh_core::{utils::path_to_string, Context};
use pjsh_eval::{execute_statement, interpolate_word};
use pjsh_parse::{parse, parse_interpolation};
use shell::context::initialized_context;
pub use shell::Shell;
use shell::{CommandShell, FileParseShell, FileShell, InteractiveShell, StdinShell};

/// Init script to always source when starting a new shell.
const INIT_ALWAYS_SCRIPT_NAME: &str = ".pjsh/init-always.pjsh";

/// Init script to source when starting an interactive shell.
const INIT_INTERACTIVE_SCRIPT_NAME: &str = ".pjsh/init-interactive.pjsh";

/// Path to the user's shell history file relative to the user's home directory.
const USER_HISTORY_FILE_NAME: &str = ".pjsh/history.txt";

/// Command line options for the application's CLI.
#[derive(Parser)]
#[clap(
    about = "A small shell for command interpretation.",
    version = crate_version!()
)]
struct Opts {
    /// Execute a command rather than a script file.
    #[clap(short = 'c', long = "command", requires = "script_file")]
    is_command: bool,

    /// Print the AST without executing it.
    #[clap(
        long = "parse",
        requires = "script_file",
        conflicts_with = "is_command"
    )]
    is_parse_only: bool,

    /// Force an interactive shell.
    #[clap(short = 'i', long = "interactive")]
    force_interactive: bool,

    /// Script file.
    script_file: Option<String>,

    /// Script arguments.
    args: Vec<String>,
}

/// Entrypoint for the application.
pub fn main() -> ExitCode {
    let mut opts = Opts::parse();
    let interactive = opts.force_interactive || !opts.is_command && opts.script_file.is_none();

    let first_arg = match &opts.is_command {
        true => current_exe().map_or_else(|_| String::from("pjsh"), path_to_string),
        false => opts
            .script_file
            .to_owned()
            .unwrap_or_else(|| String::from("pjsh")),
    };

    let mut args = Vec::with_capacity(opts.args.len() + 1);
    args.push(first_arg);
    for arg in std::mem::take(&mut opts.args) {
        args.push(arg);
    }

    let script_file = match opts.is_command {
        true => None,
        false => opts.script_file.as_ref().map(PathBuf::from),
    };

    let (context, completer) = initialized_context(args, script_file);
    let context = Arc::new(Mutex::new(context));

    source_init_scripts(interactive, &mut context.lock());

    // Not guaranteed to exit.
    let exit_code = run(&opts, Arc::clone(&context), completer);

    // If the shell exits cleanly, attempt to stop all threads and processes that it has spawned.
    let context = context.lock();
    let host = &mut context.host.lock();
    host.join_all_threads();
    host.kill_all_processes();

    exit_code
}

/// Interpolates a string using a [`Context`].
fn interpolate(src: &str, context: Arc<Mutex<Context>>) -> String {
    match parse_interpolation(src).map(|word| interpolate_word(&word, &context.lock())) {
        Ok(Ok(string)) => string,
        Ok(Err(eval_error)) => {
            eprintln!("pjsh: {}", eval_error);
            src.to_string()
        }
        Err(parse_error) => {
            eprintln!("pjsh: {}", parse_error);
            src.to_string()
        }
    }
}

/// Runs the main loop of a [`Shell`].
///
/// This method is not guaranteed to exit.
pub(crate) fn run_shell<S: Shell, E: ErrorHandler>(
    mut shell: S,
    error_handler: &E,
    context: Arc<Mutex<Context>>,
) -> ExitCode {
    if let Err(error) = shell.init() {
        error_handler.display_error(error);
        return ExitCode::FAILURE;
    }

    if let Err(error) = shell.run(Arc::clone(&context)) {
        error_handler.display_error(error);
        return ExitCode::FAILURE;
    }

    if let Err(error) = shell.exit() {
        error_handler.display_error(error);
        return ExitCode::FAILURE;
    }

    ExitCode::from(context.lock().last_exit().abs().min(u8::MAX.into()) as u8)
}

/// Runs the shell.
///
/// This method is not guaranteed to exit.
fn run(opts: &Opts, context: Arc<Mutex<Context>>, completer: Arc<Mutex<Completer>>) -> ExitCode {
    if opts.is_command {
        // The script_file argument is a command rather than a file path.
        let cmd = opts.script_file.to_owned().expect("cmd should be defined");
        return run_shell(CommandShell::new(cmd), &GuidingErrorHandler, context);
    }

    if let Some(script_file) = &opts.script_file {
        let file = File::open(script_file).expect("script file should be readable");
        return if opts.is_parse_only {
            run_shell(FileParseShell::new(file), &GuidingErrorHandler, context)
        } else {
            run_shell(FileShell::new(file), &GuidingErrorHandler, context)
        };
    }

    // Read input from stdin if stdin is not considered interactive.
    if !atty::is(atty::Stream::Stdin) {
        return run_shell(StdinShell, &GuidingErrorHandler, context);
    }

    // Construct a new interactive shell if stdin is considered interactive.
    run_shell(
        InteractiveShell::new(Arc::clone(&context), completer),
        &SimpleErrorHandler,
        context,
    )
}

/// Interrupts the currently running threads and processes in a context.
fn interrupt(context: &mut Context) {
    eprintln!("pjsh: interrupt");
    let mut host = context.host.lock();
    host.join_all_threads();
    host.kill_all_processes();
}

/// Sources all init scripts for the shell.
fn source_init_scripts(interactive: bool, context: &mut Context) {
    let mut script_names = Vec::with_capacity(2);
    script_names.push(INIT_ALWAYS_SCRIPT_NAME);

    if interactive {
        script_names.push(INIT_INTERACTIVE_SCRIPT_NAME);
    }

    let Some(home) = dirs::home_dir() else {
        return;
    };

    script_names
        .into_iter()
        .map(|script| home.join(script))
        .filter(|path| path.is_file())
        .for_each(|script| source_file(script, context));
}

/// Sources a file.
pub(crate) fn source_file(file: PathBuf, context: &mut Context) {
    let mut io = context.io();
    let Ok(file_contents) = read_to_string(&file) else {
        let _ = writeln!(io.stderr, "pjsh: file is not readable: {}", path_to_string(&file));
        return;
    };
    match parse(&file_contents, &context.aliases) {
        Ok(program) => {
            for statement in program.statements {
                let Err(error) = execute_statement(&statement, context) else {
                    continue;
                };

                let _ = writeln!(io.stderr, "pjsh: {error}");
                break;
            }
        }
        Err(error) => {
            let _ = writeln!(io.stderr, "pjsh: {error}");
        }
    }
}
