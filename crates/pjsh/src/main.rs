mod complete;
mod exec;
mod shell;

#[cfg(test)]
mod tests;

use std::{env::current_exe, path::PathBuf, sync::Arc};

use ansi_term::{Color, Style};
use clap::{crate_version, Parser};
use exec::{AstPrinter, Execute, ProgramExecutor};
use parking_lot::Mutex;
use pjsh_core::{utils::path_to_string, Context};
use pjsh_eval::interpolate_word;
use pjsh_parse::{parse, parse_interpolation, ParseError};
use shell::file_buffer_shell::FileBufferShell;
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
    about("A small shell for command interpretation."),
    version(crate_version!())
)]
struct Opts {
    /// Treat the first argument as a command rather than a script file.
    #[clap(short('c'), long("command"), requires("script_file"))]
    is_command: bool,

    /// Print the AST without executing it.
    #[clap(long("parse"), requires("script_file"))]
    is_parse_only: bool,

    /// Script file.
    script_file: Option<String>,

    /// Script arguments.
    args: Vec<String>,
}

/// Entrypoint for the application.
pub fn main() {
    let mut opts = Opts::parse();
    let interactive = !opts.is_command && opts.script_file.is_none();
    let executor: Box<dyn Execute> = match opts.is_parse_only {
        true => Box::new(AstPrinter),
        false => Box::new(ProgramExecutor),
    };

    let first_arg = match &opts.is_command {
        true => current_exe()
            .map(path_to_string)
            .unwrap_or_else(|_| String::from("pjsh")),
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

    let context = Arc::new(Mutex::new(initialized_context(
        args,
        script_file,
        interactive,
    )));

    let shell = new_shell(&opts, Arc::clone(&context));
    source_init_scripts(interactive, executor.as_ref(), Arc::clone(&context));

    run_shell(shell, executor.as_ref(), Arc::clone(&context)); // Not guaranteed to exit.

    // If the shell exits cleanly, attempt to stop all threads and processes that it has spawned.
    context.lock().host.lock().join_all_threads();
    context.lock().host.lock().kill_all_processes();
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

    let raw_ps1 = context.lock().get_var("PS1").unwrap_or("\\$ ").to_owned();
    let raw_ps2 = context.lock().get_var("PS2").unwrap_or("\\> ").to_owned();

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
) {
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
                context.lock().host.lock().eprintln("pjsh: logout");
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
                            context.lock().host.lock().eprintln("pjsh: logout");
                            break 'main;
                        }
                        shell::ShellInput::None => break,
                    };
                }

                // Unrecoverable error.
                Err(error) => {
                    eprintln!("pjsh: parse error: {}", error);
                    print_parse_error_details(&line, &error);
                    break;
                }
            }
        }
    }

    shell.save_history(history_file().as_path());
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
fn new_shell(opts: &Opts, context: Arc<Mutex<Context>>) -> Box<dyn Shell> {
    if opts.is_command {
        // The script_file argument is a command rather than a file path.
        let command = opts.script_file.to_owned().expect("command is defined");
        return Box::new(SingleCommandShell::new(command));
    }

    if let Some(script_file) = opts.script_file.to_owned() {
        return Box::new(FileBufferShell::new(script_file));
    }

    // Construct a new interactive shell if no other arguments are given.
    Box::new(RustylineShell::new(history_file().as_path(), context))
}

/// Interrupts the currently running threads and processes in a context.
fn interrupt(context: &mut Context) {
    let mut host = context.host.lock();
    host.join_all_threads();
    host.kill_all_processes();
    host.eprintln("pjsh: interrupt");
}

/// Prints process IDs (PIDs) to stderr for each child process that is managed by the shell, and
/// that have exited since last checking.
fn print_exited_child_processes(context: &mut Context) {
    let mut host = context.host.lock();
    for pid in host.take_exited_child_processes() {
        host.eprintln(&format!("pjsh: PID {} exited", pid));
    }
}

/// Sources all init scripts for the shell.
fn source_init_scripts(interactive: bool, executor: &dyn Execute, context: Arc<Mutex<Context>>) {
    let mut script_names = vec![INIT_ALWAYS_SCRIPT_NAME];

    if interactive {
        script_names.push(INIT_INTERACTIVE_SCRIPT_NAME);
    }

    for script_name in script_names {
        if let Some(init_script) = dirs::home_dir().map(|mut path| {
            path.push(script_name);
            path
        }) {
            if init_script.exists() {
                let init_shell = Box::new(FileBufferShell::new(init_script));
                run_shell(init_shell, executor, Arc::clone(&context));
            }
        }
    }
}

/// Returns a path to the current user's shell history file.
fn history_file() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(USER_HISTORY_FILE_NAME);
    path
}
