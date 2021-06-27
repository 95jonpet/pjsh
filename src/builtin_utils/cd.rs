use std::env;

use super::Builtin;
use crate::old::executor::Executor;

pub struct Cd {}

impl Builtin for Cd {
    fn execute(args: &Vec<String>, _executor: &Executor) -> Result<i32, String> {
        let borrowed_args: Vec<&str> = args.iter().map(String::as_str).collect();
        match borrowed_args.as_slice() {
            [] => {
                if let Ok(home) = env::var("HOME") {
                    env::set_current_dir(home)
                        .map_or_else(|error| Err(error.to_string()), |_| Ok(0))
                } else {
                    Ok(0)
                }
            }
            ["-"] => unimplemented!(r#"Should be equivalent to: cd "$OLDPWD" && pwd"#),
            [path] => env::set_current_dir(path.to_owned())
                .map_or_else(|error| Err(error.to_string()), |_| Ok(0)),
            _ => todo!(),
        }
    }
}
