mod executor;
mod lexer;
mod parser;
mod shell;
mod token;

use executor::Executor;
use lexer::Lexer;
use parser::{Cmd, Parser};
use shell::Shell;
use std::env;
use std::process::{Command, Stdio};

fn main() {
    let mut args = env::args();
    args.next(); // Skip first argument = path to the executable.

    let shell = Shell::new(args.last());
    for line in shell {
        let lexer = Lexer::new(&line);
        let mut parser = Parser::new(lexer.peekable());
        let cmd = parser.get();

        let executor = Executor::new();

        match cmd {
            Ok(Cmd::Single(command)) => {
                // command.execute();
                executor.execute_single(command);
            }
            _ => (),
        }
    }
}
