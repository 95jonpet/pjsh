mod builtins;
mod executor;
mod lexer;
mod parser;
mod shell;
mod token;

use executor::Executor;
use lexer::Lexer;
use parser::Parser;
use shell::Shell;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

fn main() {
    let mut args = env::args();
    args.next(); // Skip first argument = path to the executable.

    let shell = Rc::new(RefCell::new(Shell::new(args.last())));
    let executor = Executor::new();

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
