mod executor;
mod lexer;
mod parser;
mod shell;
mod token;

use executor::Executor;
use lexer::Lexer;
use parser::Parser;
use shell::Shell;
use std::env;

fn main() {
    let mut args = env::args();
    args.next(); // Skip first argument = path to the executable.

    let shell = Shell::new(args.last());
    for line in shell {
        let lexer = Lexer::new(&line);
        let mut parser = Parser::new(lexer.peekable());
        let executor = Executor::new();
        if let Ok(cmd) = parser.get() {
            executor.execute(cmd, true);
        }
    }
}
