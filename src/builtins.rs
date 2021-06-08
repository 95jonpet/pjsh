use std::{cell::RefCell, collections::HashMap, env, path::PathBuf, rc::Rc};

use crate::{executor::Executor, lexer::Lexer, parser::Parser, shell::Shell};

pub fn alias(
    aliases: &mut HashMap<String, String>,
    env: HashMap<String, String>,
    args: Vec<String>,
) -> bool {
    if args.len() > 2 {
        eprintln!("ERROR: Too many arguments.");
        return false;
    }

    if let Some((key, value)) = env.into_iter().next() {
        return aliases.insert(key, value).is_some();
    }

    if let Some(alias_name) = args.first() {
        if let Some(alias_value) = aliases.get(alias_name) {
            println!("alias {}='{:?}'", alias_name, alias_value);
            true
        } else {
            eprintln!("alias {} not found", alias_name);
            false
        }
    } else {
        let mut strings: Vec<String> = aliases
            .iter()
            .map(|(key, value)| format!("alias {}='{}'", key, value))
            .collect();
        strings.sort_unstable();
        for string in strings {
            println!("{}", string)
        }
        true
    }
}

pub fn cd(args: Vec<String>) -> bool {
    if args.is_empty() {
        true
    } else {
        let new_dir = args
            .into_iter()
            .next()
            .unwrap_or_else(|| env::var("HOME").unwrap());
        if let Err(e) = env::set_current_dir(new_dir) {
            eprintln!("ERROR: {}", e);
            false
        } else {
            true
        }
    }
}

pub fn exit(args: Vec<String>) -> bool {
    if args.len() > 1 {
        eprintln!("ERROR: Too many arguments.");
        false
    } else {
        if let Ok(code) = args.first().map_or(Ok(0), |arg| arg.parse::<i32>()) {
            std::process::exit(code)
        }

        false
    }
}

pub fn source(args: &Vec<String>, executor: &mut Executor) -> bool {
    let file = args.get(0).unwrap();
    let shell = Rc::new(RefCell::new(Shell::from_file(PathBuf::from(file))));
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
                    return false;
                }
            }
        } else {
            break;
        }
    }
    true
}
