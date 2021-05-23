mod lexer;
mod parser;
mod shell;

use lexer::{Lexer, Token};
use parser::Parser;
use shell::Shell;
use std::env;
use std::process::Stdio;

fn main() {
    let mut args = env::args();
    args.next(); // Skip first argument = path to the executable.

    let shell = Shell::new(args.last());
    // let input = read_input(env::args());
    // TODO Read input.
    // TODO Tokenize input.
    // TODO Parse input into commands.
    // TODO Perform command expansion.
    // TODO Perform redirection.
    // TODO Execute function.
    // TODO Wait for completion and collect exit status.

    for line in shell {
        let mut lexer = Lexer::new(&line);
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(token) = lexer.next_token() {
            tokens.push(token);
        }

        // println!("{:?}", tokens);
        let parser = Parser::new(tokens);
        let commands = parser.parse();

        for mut command in commands {
            command
                .stderr(Stdio::inherit())
                .stdout(Stdio::inherit())
                .output()
                .expect(format!("Command failed: {:?}", command).as_str());
        }

        // println!("{:?}", commands);
    }
}
