mod ast;
mod cursor;
mod executor;
mod input;
mod lexer;
mod parser;
mod token;

use clap::{crate_name, crate_version, Clap};
use input::InputLines;
use std::io::BufReader;
use std::path::PathBuf;
use std::{env, fs, io};

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
    let input = match cli {
        conf if conf.command.is_some() => InputLines::Single(conf.command),
        conf if conf.script_file.is_some() => InputLines::Buffered(Box::new(BufReader::new(
            fs::File::open(conf.script_file.unwrap()).unwrap(),
        ))),
        _ => InputLines::Buffered(Box::new(BufReader::new(io::stdin()))),
    };
    let cursor = crate::cursor::Cursor::new(input, interactive);
    let lexer = crate::lexer::Lexer::new(cursor);
    let mut parser = crate::parser::Parser::new(Box::new(lexer));
    let executor = crate::executor::Executor::new();

    loop {
        if let Ok(program) = parser.parse() {
            let result = executor.execute(program);
            match result {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }
}
