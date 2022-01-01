use std::{path::PathBuf, sync::Arc};

use parking_lot::Mutex;
use pjsh_core::{
    utils::{path_to_string, resolve_path},
    Context, InternalCommand, InternalIo,
};

use crate::status;

pub struct Cd;

impl Cd {
    fn change_directory(&self, path: PathBuf, context: &mut Context) -> bool {
        if !path.is_dir() {
            return false;
        }

        self.update_old_pwd(context);

        context
            .scope
            .set_env(String::from("PWD"), path_to_string(&path));
        let _ = std::env::set_current_dir(path);

        true
    }

    /// Sets `$OLDPWD` to `$PWD` in a context.
    fn update_old_pwd(&self, context: &mut Context) {
        if let Some(pwd) = context.scope.get_env("PWD") {
            context.scope.set_env(String::from("OLDPWD"), pwd);
        }
    }
}

impl InternalCommand for Cd {
    fn name(&self) -> &str {
        "cd"
    }

    fn run(
        &self,
        args: &[String],
        context: Arc<Mutex<Context>>,
        io: Arc<Mutex<InternalIo>>,
    ) -> i32 {
        match args {
            [] => {
                let path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
                self.change_directory(path, &mut context.lock());
                status::SUCCESS
            }
            [target] if target == "-" => match context.lock().scope.get_env("OLDPWD") {
                Some(oldpwd) => {
                    let path = PathBuf::from(oldpwd);
                    self.change_directory(path, &mut context.lock());
                    let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
                    match writeln!(io.lock().stdout, "{}", path_to_string(&path)) {
                        Ok(_) => status::SUCCESS,
                        Err(error) => {
                            let _ = writeln!(
                                io.lock().stderr,
                                "cd: could not write path to stdout: {}",
                                error
                            );
                            status::GENERAL_ERROR
                        }
                    }
                }
                None => {
                    let _ = writeln!(io.lock().stderr, "cd: OLDPWD not set");
                    status::GENERAL_ERROR
                }
            },
            [target] => {
                let path = resolve_path(&context.lock(), target);
                self.change_directory(path, &mut context.lock());
                status::SUCCESS
            }
            _ => {
                let _ = writeln!(
                    io.lock().stderr,
                    "cd: invalid arguments: {}",
                    args.join(" ")
                );
                status::BUILTIN_ERROR
            }
        }
    }
}

pub struct Pwd;

impl InternalCommand for Pwd {
    fn name(&self) -> &str {
        "pwd"
    }

    fn run(
        &self,
        _args: &[String],
        _context: Arc<Mutex<Context>>,
        io: Arc<Mutex<InternalIo>>,
    ) -> i32 {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        match writeln!(io.lock().stdout, "{}", path_to_string(&path)) {
            Ok(_) => status::SUCCESS,
            Err(error) => {
                let _ = writeln!(
                    io.lock().stderr,
                    "pwd: could not write path to stdout: {}",
                    error
                );
                status::GENERAL_ERROR
            }
        }
    }
}
