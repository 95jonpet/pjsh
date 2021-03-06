mod command_shell;
mod completion;
mod exec;
mod file_shell;
mod init;
mod interactive_shell;
mod shell;

#[cfg(test)]
mod tests;

use std::{path::PathBuf, sync::Arc};

use clap::{crate_version, Parser};
use command_shell::SingleCommandShell;
use exec::create_executor;
use file_shell::FileBufferShell;
use init::initialized_context;
use interactive_shell::RustylineShell;
use parking_lot::Mutex;
use pjsh_core::Context;
use pjsh_exec::{interpolate_word, Executor, FileDescriptors};
use pjsh_parse::{parse, parse_interpolation, ParseError};
use shell::Shell;

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
    /// Command
    #[clap(short, long, conflicts_with("input"))]
    command: Option<String>,

    /// Input file
    #[clap(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Script arguments
    args: Vec<String>,
}

/// Entrypoint for the application.
pub fn main() {
    let opts = Opts::parse();

    let shell: Box<dyn Shell> = match opts.input {
        Some(script_file) => Box::new(FileBufferShell::new(script_file)),
        None if opts.command.is_some() => Box::new(SingleCommandShell::new(opts.command.unwrap())),
        _ => Box::new(RustylineShell::new(history_file().as_path())),
    };
    let context = Arc::new(Mutex::new(initialized_context(shell.is_interactive())));

    source_init_scripts(shell.is_interactive(), Arc::clone(&context));

    run_shell(shell, Arc::clone(&context)); // Not guaranteed to exit.

    // If the shell exits cleanly, attempt to stop all threads and processes that it has spawned.
    context.lock().host.lock().join_all_threads();
    context.lock().host.lock().kill_all_processes();
}

/// Interpolates a string using a [`Context`].
fn interpolate(src: &str, context: Arc<Mutex<Context>>, executor: &Executor) -> String {
    match parse_interpolation(src).map(|word| interpolate_word(executor, word, context)) {
        Ok(string) => string,
        Err(error) => {
            eprintln!("pjsh: {}", error);
            src.to_string()
        }
    }
}

/// Get interpolated PS1 and PS2 prompts from a context.
fn get_prompts(
    interactive: bool,
    context: Arc<Mutex<Context>>,
    executor: &Executor,
) -> (String, String) {
    if !interactive {
        return (String::default(), String::default());
    }

    let raw_ps1 = context.lock().get_var("PS1").unwrap_or("\\$ ").to_owned();
    let raw_ps2 = context.lock().get_var("PS2").unwrap_or("\\> ").to_owned();

    let ps1 = interpolate(&raw_ps1, Arc::clone(&context), executor);
    let ps2 = interpolate(&raw_ps2, Arc::clone(&context), executor);

    (ps1, ps2)
}

/// Main loop for running a [`Shell`].
///
/// This method is not guaranteed to exit.
pub(crate) fn run_shell(mut shell: Box<dyn Shell>, context: Arc<Mutex<Context>>) {
    let executor = create_executor();
    'main: loop {
        let (ps1, ps2) = get_prompts(shell.is_interactive(), Arc::clone(&context), &executor);
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
            match parse(&line) {
                // If a valid program can be parsed from the buffer, execute it.
                Ok(program) => {
                    shell.add_history_entry(line.trim());
                    for statement in program.statements {
                        let fds = FileDescriptors::new();
                        executor.execute_statement(statement, Arc::clone(&context), &fds);
                    }
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
                    break;
                }
            }
        }
    }

    shell.save_history(history_file().as_path());
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
fn source_init_scripts(interactive: bool, context: Arc<Mutex<Context>>) {
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
                run_shell(init_shell, Arc::clone(&context));
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
