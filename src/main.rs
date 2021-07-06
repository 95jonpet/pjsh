mod ast;
mod cursor;
mod executor;
mod input;
mod lexer;
mod parser;
mod token;

use clap::{crate_name, crate_version, Clap};
use std::io::BufReader;
use std::path::PathBuf;
use std::{env, io};

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
    let input = crate::input::InputLines::Buffered(Box::new(BufReader::new(io::stdin())));
    let cursor = crate::cursor::Cursor::new(input, true);
    let lexer = crate::lexer::Lexer::new(cursor);
    let mut parser = crate::parser::Parser::new(Box::new(lexer));
    let executor = crate::executor::Executor::new();

    loop {
        if let Ok(program) = parser.parse() {
            let result = executor.execute(program);
            match result {
                Ok(_) => (),
                Err(_) => println!("Execution failed."),
            }
        }
    }
}
