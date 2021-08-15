use std::{
    cell::RefCell,
    env,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    execution::{
        environment::{path_to_lossy_string, Environment},
        exit_status::ExitStatus,
    },
    options::Options,
};

use super::Builtin;

pub(crate) struct Cd;
impl Cd {
    fn set_current_dir<P>(directory: P, env: &mut dyn Environment) -> ExitStatus
    where
        P: AsRef<Path>,
    {
        let current_dir = env.var("PWD");
        let path = current_dir
            .map(PathBuf::from)
            .unwrap_or_default()
            .join(directory)
            .canonicalize();

        if let Err(error) = path {
            eprintln!("pjsh: cd: {}", error);
            return ExitStatus::new(1);
        }

        let resolved_path = path.unwrap();
        match env::set_current_dir(&resolved_path) {
            Ok(()) => {
                env.set_var(String::from("OLDPWD"), env.var("PWD").unwrap_or_default());
                env.set_var(String::from("PWD"), path_to_lossy_string(resolved_path));
                ExitStatus::SUCCESS
            }
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
        env: &mut dyn Environment,
    ) -> crate::execution::exit_status::ExitStatus {
        match args {
            [path] => Self::set_current_dir(path, env),
            [] => {
                if let Some(home) = env.var("HOME") {
                    Self::set_current_dir(home, env)
                } else {
                    ExitStatus::new(1)
                }
            }
            _ => ExitStatus::new(1),
        }
    }
}

pub(crate) struct Exit;
impl Builtin for Exit {
    fn execute(&self, args: &[String], _env: &mut dyn Environment) -> ExitStatus {
        match args {
            [code_str] => {
                if let Ok(code) = code_str.parse() {
                    return ExitStatus::new(code);
                }

                ExitStatus::new(1)
            }
            [] => ExitStatus::SUCCESS,
            _ => ExitStatus::new(1),
        }
    }
}

pub(crate) struct Set {
    options: Rc<RefCell<Options>>,
}
impl Set {
    pub(crate) fn new(options: Rc<RefCell<Options>>) -> Self {
        Self { options }
    }
}
impl Builtin for Set {
    fn execute(&self, args: &[String], _env: &mut dyn Environment) -> ExitStatus {
        let command_args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        match command_args.as_slice() {
            ["-o", "xlex"] => self.options.borrow_mut().debug_lexing = true,
            ["-o", "xparse"] => self.options.borrow_mut().debug_parsing = true,
            ["-v"] | ["-o", "verbose"] => self.options.borrow_mut().print_input = true,
            args => {
                eprintln!("set: unknown arguments {:?}", args);
                return ExitStatus::new(1);
            }
        }
        ExitStatus::SUCCESS
    }
}

pub(crate) struct Unset;
impl Builtin for Unset {
    fn execute(&self, args: &[String], env: &mut dyn Environment) -> ExitStatus {
        for variable_name in args {
            env.unset_var(variable_name);
        }

        ExitStatus::SUCCESS
    }
}

pub(crate) struct Which;
impl Builtin for Which {
    fn execute(&self, args: &[String], env: &mut dyn Environment) -> ExitStatus {
        match args {
            [program] => {
                if let Some(path) = env.find_program(program) {
                    println!("{}", path_to_lossy_string(path));
                    ExitStatus::SUCCESS
                } else {
                    eprintln!(
                        "which: no {} in ({})",
                        program,
                        env.var("PATH").unwrap_or_default()
                    );
                    ExitStatus::new(1)
                }
            }
            args => {
                eprintln!("set: unknown arguments {:?}", args);
                ExitStatus::new(1)
            }
        }
    }
}
