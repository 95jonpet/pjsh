use std::{env, path::Path};

use crate::executor::exit_status::ExitStatus;

use super::Builtin;

pub(crate) struct Cd;

impl Cd {
    pub fn new() -> Self {
        Self {}
    }

    fn set_current_dir<P>(path: P) -> ExitStatus
    where
        P: AsRef<Path>,
    {
        if env::set_current_dir(path).is_ok() {
            ExitStatus::new(0)
        } else {
            ExitStatus::new(1)
        }
    }
}

impl Builtin for Cd {
    fn execute(&self, args: &Vec<String>) -> crate::executor::exit_status::ExitStatus {
        match &args[..] {
            [path] => Self::set_current_dir(path),
            [] => Self::set_current_dir(env::var("HOME").unwrap()),
            _ => ExitStatus::new(0),
        }
    }
}
