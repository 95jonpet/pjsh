use std::{collections::HashMap, env};

pub fn alias(aliases: &mut HashMap<String, Vec<String>>, args: Vec<String>) -> bool {
    if args.len() > 2 {
        eprintln!("ERROR: Too many arguments.");
        return false;
    }

    if let Some(alias_name) = args.first() {
        if let Some(alias_value) = args.get(1) {
            aliases
                .insert(alias_name.to_owned(), vec![alias_value.to_owned()])
                .is_some()
        } else {
            if let Some(alias_value) = aliases.get(alias_name) {
                println!("alias {}='{:?}'", alias_name, alias_value);
                true
            } else {
                eprintln!("alias {} not found", alias_name);
                false
            }
        }
    } else {
        eprintln!("ERROR: missing alias name.");
        false
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
