mod ast;
mod builtin;
mod cursor;
mod execution;
mod input;
mod lexer;
pub(crate) mod options;
mod parser;
mod token;

use clap::{crate_name, crate_version, Clap};
use cursor::Cursor;
use execution::Executor;
use input::InputLines;
use lexer::Lexer;
use options::Options;
use parser::posix::PosixParser;
use std::cell::RefCell;
use std::io::BufReader;
use std::path::PathBuf;
use std::rc::Rc;
use std::{env, fs, io};

use crate::ast::{CompleteCommands, Program};
use crate::cursor::PS1;

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
    let cli = Cli::parse();
    let interactive = cli.command.is_none() && cli.script_file.is_none();
    let options = Rc::new(RefCell::new(Options::default()));
    let input = match cli {
        conf if conf.command.is_some() => InputLines::Single(conf.command),
        conf if conf.script_file.is_some() => InputLines::Buffered(Box::new(BufReader::new(
            fs::File::open(conf.script_file.unwrap()).unwrap(),
        ))),
        _ => InputLines::Buffered(Box::new(BufReader::new(io::stdin()))),
    };
    let cursor = Rc::new(RefCell::new(Cursor::new(
        input,
        interactive,
        options.clone(),
    )));
    let lexer = Lexer::new(cursor.clone(), options.clone());
    let mut parser = PosixParser::new(Box::new(lexer), options.clone());
    let executor = Executor::new(options);

    // In interactive mode, multiple programs are accepted - typically one for each line of input.
    // In non-interactive mode, only one program, consisting of all input, should be accepted.
    loop {
        cursor.borrow_mut().advance_line(PS1);

        if interactive {
            match parser.parse_complete_command() {
                Ok(complete_command) => {
                    let program = Program(CompleteCommands(vec![complete_command]));
                    if let Err(exec_error) = executor.execute(program) {
                        eprintln!("pjsh: {}", exec_error);
                    }
                }
                Err(parse_error) => eprintln!("pjsh: {}", parse_error),
            }
        } else {
            match parser.parse_program() {
                Ok(program) => {
                    let result = executor.execute(program);
                    match result {
                        Ok(_) => (),
                        Err(exec_error) => eprintln!("pjsh: {}", exec_error),
                    }
                }
                Err(parse_error) => eprintln!("pjsh: {}", parse_error),
            }

            // Non-interactive mode. Don't loop.
            break;
        }
    }
}
