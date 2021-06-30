mod builtin_utils;
// mod builtins;
mod cursor;
mod input;
mod lexer;
mod old;
mod parser;
mod token;

use old::executor::Executor;
use old::lexer::Lexer;
use old::parser::Cmd;
use old::parser::Parser;
use old::shell::Shell;

use clap::{crate_name, crate_version, Clap};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::env::VarError;
use std::path::PathBuf;
use std::rc::Rc;

use crate::old::parser::Io;
use crate::old::parser::SimpleCommand;

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
    let mut executor = Executor::new(Rc::clone(&shell));
    executor.execute(perform_login(), false);

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

fn perform_login() -> Cmd {
    if let Ok(home_dir) = home_dir() {
        let login_script_path: PathBuf = [&home_dir, ".pjshrc"].iter().collect();

        if !login_script_path.is_file() {
            return Cmd::NoOp;
        }

        if let Some(login_script_path_string) = login_script_path.to_str() {
            return Cmd::Simple(SimpleCommand::new(
                String::from("source"),
                vec![login_script_path_string.to_owned()],
                Io::new(),
                HashMap::new(),
            ));
        }
    }

    Cmd::NoOp
}

fn home_dir() -> Result<String, VarError> {
    env::var("HOME").or_else(|_| {
        let drive = env::var("HOMEDRIVE")?;
        let path = env::var("HOMEPATH")?;

        let mut home = drive;
        home.push_str(&path);
        home = home.replace("\\", "/");

        Ok(home)
    })
}

fn create_shell(args: Cli) -> Rc<RefCell<Shell>> {
    let shell = match args {
        conf if conf.command.is_some() => Shell::from_command(conf.command.unwrap()),
        conf if conf.script_file.is_some() => Shell::from_file(conf.script_file.unwrap()),
        _ => Shell::interactive(),
    };

    Rc::new(RefCell::new(shell))
}
