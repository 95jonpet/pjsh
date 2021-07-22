use std::{
    borrow::BorrowMut,
    env,
    path::{Path, PathBuf},
};

use crate::execution::{environment::ExecutionEnvironment, exit_status::ExitStatus};

use super::Builtin;

pub(super) struct Cd;
impl Cd {
    fn set_current_dir<P>(directory: P) -> ExitStatus
    where
        P: AsRef<Path>,
    {
        let path = PathBuf::from(env::current_dir().unwrap())
            .join(directory)
            .canonicalize()
            .unwrap();
        if env::set_current_dir(path.clone()).is_ok() {
            ExitStatus::new(0)
        } else {
            ExitStatus::new(1)
        }
    }
}

impl Builtin for Cd {
    fn execute(
        &self,
        args: &Vec<String>,
        _env: &mut ExecutionEnvironment,
    ) -> crate::execution::exit_status::ExitStatus {
        match &args[..] {
            [path] => Self::set_current_dir(path),
            [] => Self::set_current_dir(env::var("HOME").unwrap()),
            _ => ExitStatus::new(0),
        }
    }
}

pub(super) struct Exit;
impl Builtin for Exit {
    fn execute(&self, args: &Vec<String>, _env: &mut ExecutionEnvironment) -> ExitStatus {
        match &args[..] {
            [code_str] => {
                if let Ok(code) = code_str.parse() {
                    return ExitStatus::new(code);
                }

                ExitStatus::new(1)
            }
            [] => ExitStatus::new(0),
            _ => ExitStatus::new(1),
        }
    }
}

pub(super) struct Unset;
impl Builtin for Unset {
    fn execute(&self, args: &Vec<String>, env: &mut ExecutionEnvironment) -> ExitStatus {
        for variable_name in args {
            env.borrow_mut().unset_var(variable_name);
        }

        ExitStatus::new(0)
    }
}
