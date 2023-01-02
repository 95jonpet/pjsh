mod complete;
mod exec;
mod shell;

#[cfg(test)]
mod tests;

use std::fs::{read_to_string, File};
use std::process::ExitCode;
use std::{env::current_exe, path::PathBuf, sync::Arc};

use ansi_term::{Color, Style};
use clap::{crate_version, Parser};
use exec::{AstPrinter, Execute, ProgramExecutor};
use parking_lot::Mutex;
use pjsh_core::utils::word_var;
use pjsh_core::Completions;
use pjsh_core::{utils::path_to_string, Context};
use pjsh_eval::{execute_statement, interpolate_word};
use pjsh_parse::{parse, parse_interpolation, ParseError};
use shell::input_shell::InputShell;
use shell::interactive::RustylineShell;
use shell::Shell;
use shell::{context::initialized_context, single_command_shell::SingleCommandShell};

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
    #[clap(long = "parse", requires = "script_file")]
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
    let executor: Box<dyn Execute> = match opts.is_parse_only {
        true => Box::new(AstPrinter),
        false => Box::new(ProgramExecutor::new(!interactive)),
    };

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

    let (context, completions) = initialized_context(args, script_file);
    let context = Arc::new(Mutex::new(context));

    source_init_scripts(interactive, &mut context.lock());
    let shell = new_shell(&opts, Arc::clone(&context), completions);

    // Not guaranteed to exit.
    let code = run_shell(shell, executor.as_ref(), Arc::clone(&context));

    // If the shell exits cleanly, attempt to stop all threads and processes that it has spawned.
    context.lock().host.lock().join_all_threads();
    context.lock().host.lock().kill_all_processes();

    code
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

/// Get interpolated PS1 and PS2 prompts from a context.
fn get_prompts(interactive: bool, context: Arc<Mutex<Context>>) -> (String, String) {
    if !interactive {
        return (String::default(), String::default());
    }

    let raw_ps1 = word_var(&context.lock(), "PS1")
        .unwrap_or("\\$ ")
        .to_owned();
    let raw_ps2 = word_var(&context.lock(), "PS2")
        .unwrap_or("\\> ")
        .to_owned();

    let ps1 = interpolate(&raw_ps1, Arc::clone(&context));
    let ps2 = interpolate(&raw_ps2, Arc::clone(&context));

    (ps1, ps2)
}

/// Main loop for running a [`Shell`].
///
/// This method is not guaranteed to exit.
pub(crate) fn run_shell(
    mut shell: Box<dyn Shell>,
    executor: &dyn Execute,
    context: Arc<Mutex<Context>>,
) -> ExitCode {
    let mut exit_code = ExitCode::SUCCESS;
    'main: loop {
        let (ps1, ps2) = get_prompts(shell.is_interactive(), Arc::clone(&context));
        print_exited_child_processes(&mut context.lock());

        let mut line = match shell.prompt_line(&ps1) {
            shell::ShellInput::Line(line) => line,
            shell::ShellInput::Interrupt => {
                interrupt(&mut context.lock());
                continue;
            }
            shell::ShellInput::Logout => {
                eprintln!("pjsh: logout");
                break 'main;
            }
            shell::ShellInput::None => break,
        };

        // Repeatedly ask for lines of input until a valid program can be executed.
        loop {
            let aliases = context.lock().aliases.clone();
            match parse(&line, &aliases) {
                // If a valid program can be parsed from the buffer, execute it.
                Ok(program) => {
                    shell.add_history_entry(line.trim());
                    executor.execute(program, Arc::clone(&context));
                    break;
                }

                // If more input is required, prompt for more input and loop again.
                // The next line of input will be appended to the buffer and parsed.
                Err(ParseError::IncompleteSequence | ParseError::UnexpectedEof) => {
                    match shell.prompt_line(&ps2) {
                        shell::ShellInput::Line(next_line) => line.push_str(&next_line),
                        shell::ShellInput::Interrupt => {
                            interrupt(&mut context.lock());
                            continue 'main;
                        }
                        shell::ShellInput::Logout => {
                            eprintln!("pjsh: logout");
                            break 'main;
                        }
                        shell::ShellInput::None => break,
                    };
                }

                // Unrecoverable error.
                Err(error) => {
                    eprintln!("pjsh: parse error: {}", error);
                    print_parse_error_details(&line, &error);
                    if shell.is_interactive() {
                        break;
                    } else {
                        exit_code = ExitCode::FAILURE;
                        break 'main;
                    }
                }
            }
        }

        // Execution errors are not tolerated within non-interactive shells.
        // Thus, execution should be terminated.
        if context.lock().last_exit() != 0 && !shell.is_interactive() {
            exit_code = ExitCode::from(context.lock().last_exit().abs().min(u8::MAX.into()) as u8);
            break;
        }
    }

    shell.save_history(history_file().as_path());
    exit_code
}

/// Prints details related to a parse error.
fn print_parse_error_details(line: &str, error: &ParseError) {
    if let Some(span) = error.span() {
        let marked_line_start = line[..span.start].rfind('\n').unwrap_or(0);
        let marked_start = span.start - marked_line_start;

        let marker_indent = " ".repeat(marked_start);
        let mut marker = marker_indent + &("^".repeat(span.end - span.start));
        marker.push_str(" help: ");
        marker.push_str(error.help());

        eprintln!("{line}{}", Style::new().fg(Color::Red).paint(&marker));
    }
}

/// Constructs a new shell.
fn new_shell(
    opts: &Opts,
    context: Arc<Mutex<Context>>,
    completions: Arc<Mutex<Completions>>,
) -> Box<dyn Shell> {
    if opts.is_command {
        // The script_file argument is a command rather than a file path.
        let command = opts.script_file.to_owned().expect("command is defined");
        return Box::new(SingleCommandShell::new(command));
    }

    if let Some(script_file) = opts.script_file.to_owned() {
        let file = File::open(script_file).expect("script file should be readable");
        return Box::new(InputShell::new(file));
    }

    // Read input from stdin if stdin is not considered interactive.
    if !atty::is(atty::Stream::Stdin) {
        return Box::new(InputShell::new(std::io::stdin()));
    }

    // Construct a new interactive shell if stdin is considered interactive.
    Box::new(RustylineShell::new(
        history_file().as_path(),
        context,
        completions,
    ))
}

/// Interrupts the currently running threads and processes in a context.
fn interrupt(context: &mut Context) {
    eprintln!("pjsh: interrupt");
    let mut host = context.host.lock();
    host.join_all_threads();
    host.kill_all_processes();
}

/// Prints process IDs (PIDs) to stderr for each child process that is managed by the shell, and
/// that have exited since last checking.
fn print_exited_child_processes(context: &mut Context) {
    let mut host = context.host.lock();
    for pid in host.take_exited_child_processes() {
        eprintln!("pjsh: PID {pid} exited");
    }
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

/// Returns a path to the current user's shell history file.
fn history_file() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(USER_HISTORY_FILE_NAME);
    path
}
