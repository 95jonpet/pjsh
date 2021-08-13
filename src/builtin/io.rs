use std::{borrow::BorrowMut, env, path::Path};

use crate::execution::{environment::Environment, exit_status::ExitStatus};

use super::Builtin;

pub(crate) struct Cd;
impl Cd {
    fn set_current_dir<P>(directory: P) -> ExitStatus
    where
        P: AsRef<Path>,
    {
        let path = env::current_dir().unwrap().join(directory).canonicalize();

        if let Err(error) = path {
            eprintln!("pjsh: cd: {}", error);
            return ExitStatus::new(1);
        }

        match env::set_current_dir(path.unwrap()) {
            Ok(()) => ExitStatus::new(0),
            Err(error) => {
                eprintln!("pjsh: cd: {}", error);
                ExitStatus::new(1)
            }
        }
    }
}

impl Builtin for Cd {
    fn execute(
        &self,
        args: &[String],
        _env: &mut impl Environment,
    ) -> crate::execution::exit_status::ExitStatus {
        match args {
            [path] => Self::set_current_dir(path),
            [] => Self::set_current_dir(env::var("HOME").unwrap()),
            _ => ExitStatus::new(0),
        }
    }
}

pub(crate) struct Exit;
impl Builtin for Exit {
    fn execute(&self, args: &[String], _env: &mut impl Environment) -> ExitStatus {
        match args {
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

pub(crate) struct Unset;
impl Builtin for Unset {
    fn execute(&self, args: &[String], env: &mut impl Environment) -> ExitStatus {
        for variable_name in args {
            env.borrow_mut().unset_var(variable_name);
        }

        ExitStatus::new(0)
    }
}
