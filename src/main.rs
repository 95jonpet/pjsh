mod builtins;
mod executor;
mod lexer;
mod parser;
mod shell;
mod token;

use executor::Executor;
use lexer::Lexer;
use parser::Cmd;
use parser::Parser;
use parser::SingleCommand;
use shell::Shell;

use clap::{crate_name, crate_version, Clap};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;

use crate::parser::Io;

/// A shell for executing POSIX commands.
#[derive(Clap, Debug)]
#[clap(name = crate_name!(), version = crate_version!())]
struct Cli {
    /// The command to execute.
    #[clap(short)]
    command: Option<String>,

    /// The path to a script which should be executed.
    #[clap(parse(from_os_str))]
    script_file: Option<PathBuf>,
}

fn main() {
    let args = Cli::parse();

    let shell = create_shell(args);
    let mut executor = Executor::new();
    executor.execute(login_command(), false);

    loop {
        let input = shell.borrow_mut().next();
        if let Some(line) = input {
            let lexer = Lexer::new(&line, Rc::clone(&shell));
            let mut parser = Parser::new(lexer, Rc::clone(&shell));
            match parser.get() {
                Ok(command) => {
                    executor.execute(command, false);
                }
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                }
            }
        } else {
            if shell.borrow().is_interactive() {
                println!();
            }
            break;
        }
    }
}

fn login_command() -> Cmd {
    let login_script_path: PathBuf = [env::var("HOME").unwrap().as_str(), ".pjshrc"]
        .iter()
        .collect();

    if !login_script_path.is_file() {
        return Cmd::NoOp;
    }

    Cmd::Single(SingleCommand::new(
        ".".to_string(),
        vec![login_script_path
            .to_str()
            .expect("login script path exists")
            .to_owned()],
        Io::new(),
        HashMap::new(),
    ))
}

fn create_shell(args: Cli) -> Rc<RefCell<Shell>> {
    let shell = match args {
        conf if conf.command.is_some() => Shell::from_command(conf.command.unwrap()),
        conf if conf.script_file.is_some() => Shell::from_file(conf.script_file.unwrap()),
        _ => Shell::interactive(),
    };

    Rc::new(RefCell::new(shell))
}
