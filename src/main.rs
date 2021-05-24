mod lexer;
mod parser;
mod shell;
mod token;

use lexer::Lexer;
use parser::Parser;
use shell::Shell;
use std::env;
use std::process::{Command, Stdio};

fn main() {
    let mut args = env::args();
    args.next(); // Skip first argument = path to the executable.

    let shell = Shell::new(args.last());
    for line in shell {
        let lexer = Lexer::new(&line);

        let parser = Parser::new(lexer.peekable());
        let commands = parser.parse();

        for command in commands {
            execute_command(command);
        }
    }
}

fn execute_command(mut command: Command) {
    let maybe_output = command
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output();

    match maybe_output {
        Ok(output) if !output.status.success() => {
            eprintln!(
                "ERROR: Command failed with status {}.",
                output.status.code().unwrap()
            );
        }
        Err(error) => eprintln!("ERROR: {}", error),
        _ => (),
    }
}
