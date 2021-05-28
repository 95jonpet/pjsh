use std::env;

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
