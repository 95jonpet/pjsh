mod init;
mod shell;

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use clap::{crate_version, Parser};
use init::init_context;
use pjsh_core::Context;
use pjsh_exec::{interpolate_word, Executor};
use pjsh_parse::{parse, parse_interpolation, ParseError};
use shell::{
    command::SingleCommandShell, file::FileBufferShell, interactive::RustylineShell, Shell,
};

/// Init script to always source when starting a new shell.
const INIT_ALWAYS_SCRIPT_NAME: &str = ".pjsh/init-always.pjsh";

/// Init script to source when starting an interactive shell.
const INIT_INTERACTIVE_SCRIPT_NAME: &str = ".pjsh/init-interactive.pjsh";

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
}

/// Entrypoint for the application.
pub fn main() {
    let opts = Opts::parse();

    let shell: Box<dyn Shell> = match opts.input {
        Some(script_file) => Box::new(FileBufferShell::new(script_file)),
        None if opts.command.is_some() => Box::new(SingleCommandShell::new(opts.command.unwrap())),
        _ => Box::new(RustylineShell::new()),
    };
    let context = Rc::new(RefCell::new(init_context()));

    source_init_scripts(shell.is_interactive(), Rc::clone(&context));

    run_shell(shell, context) // Not guaranteed to exit.
}

/// Interpolates a string using a [`Context`].
fn interpolate(src: &str, context: &Context) -> String {
    match parse_interpolation(src).map(|word| interpolate_word(word, context)) {
        Ok(string) => string,
        Err(error) => {
            eprintln!("pjsh: {}", error);
            src.to_string()
        }
    }
}

/// Get interpolated PS1 and PS2 prompts from a context.
fn get_prompts(interactive: bool, context: &Context) -> (String, String) {
    if !interactive {
        return (String::new(), String::new());
    }

    let ps1 = interpolate(
        &context
            .scope
            .get_env("PS1")
            .unwrap_or_else(|| String::from("$ ")),
        context,
    );
    let ps2 = interpolate(
        &context
            .scope
            .get_env("PS2")
            .unwrap_or_else(|| String::from("> ")),
        context,
    );

    (ps1, ps2)
}

/// Main loop for running a [`Shell`]. This method is not guaranteed to exit.
fn run_shell(mut shell: Box<dyn Shell>, context: Rc<RefCell<Context>>) {
    let executor = Executor::default();
    loop {
        let (ps1, ps2) = get_prompts(shell.is_interactive(), &context.borrow());
        print_exited_child_processes(&mut context.borrow_mut());

        // Prompt for initial input.
        let maybe_line = shell.prompt_line(&ps1);
        if maybe_line.is_none() {
            break;
        }
        let mut line = maybe_line.expect("there is one more line of input");

        // Repeatedly ask for lines of input until a valid program can be executed.
        loop {
            match parse(&line) {
                // If a valid program can be parsed from the buffer, execute it.
                Ok(program) => {
                    shell.add_history_entry(line.trim());
                    for statement in program.statements {
                        executor.execute_statement(statement, Rc::clone(&context));
                    }
                    break;
                }

                // If more input is required, prompt for more input and loop again.
                // The next line of input will be appended to the buffer and parsed.
                Err(ParseError::IncompleteSequence | ParseError::UnexpectedEof) => {
                    if let Some(next_line) = shell.prompt_line(&ps2) {
                        line.push_str(&next_line);
                    } else {
                        eprintln!("Unexpected EOF");
                        std::process::exit(1);
                    }
                }

                // Unrecoverable error.
                Err(error) => {
                    eprintln!("Parse error: {}", error);
                    break;
                }
            }
        }
    }
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
fn source_init_scripts(interactive: bool, context: Rc<RefCell<Context>>) {
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
                run_shell(init_shell, Rc::clone(&context));
            }
        }
    }
}
