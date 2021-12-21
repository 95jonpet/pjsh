use std::path::PathBuf;

use pjsh_core::{
    utils::{path_to_string, resolve_path},
    Context, InternalCommand,
};

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

    fn run(&self, args: &[String], context: &mut Context, io: &mut pjsh_core::InternalIo) -> i32 {
        match args {
            [] => {
                let path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
                self.change_directory(path, context);
                0
            }
            [target] if target == "-" => match context.scope.get_env("OLDPWD") {
                Some(oldpwd) => {
                    let path = PathBuf::from(oldpwd);
                    self.change_directory(path, context);
                    let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
                    match writeln!(io.stdout, "{}", path_to_string(&path)) {
                        Ok(_) => 0,
                        Err(error) => {
                            let _ = writeln!(
                                io.stderr,
                                "cd: could not write path to stdout: {}",
                                error
                            );
                            1
                        }
                    }
                }
                None => {
                    let _ = writeln!(io.stderr, "cd: OLDPWD not set");
                    1
                }
            },
            [target] => {
                let path = resolve_path(context, target);
                self.change_directory(path, context);
                0
            }
            _ => {
                let _ = writeln!(io.stderr, "cd: invalid arguments: {}", args.join(" "));
                2
            }
        }
    }
}

pub struct Pwd;

impl InternalCommand for Pwd {
    fn name(&self) -> &str {
        "pwd"
    }

    fn run(&self, _args: &[String], _context: &mut Context, io: &mut pjsh_core::InternalIo) -> i32 {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        match writeln!(io.stdout, "{}", path_to_string(&path)) {
            Ok(_) => 0,
            Err(error) => {
                let _ = writeln!(io.stderr, "pwd: could not write path to stdout: {}", error);
                1
            }
        }
    }
}
