use std::path::PathBuf;

use pjsh_core::{utils::path_to_string, BuiltinCommand, Context, ExecError, Result, Value};

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

    fn update_old_pwd(&self, context: &mut Context) {
        let pwd = std::env::current_dir()
            .map(|path| path_to_string(&path))
            .unwrap_or_else(|_| String::from("/"));
        context.scope.set_env(String::from("OLDPWD"), pwd);
    }
}

impl BuiltinCommand for Cd {
    fn name(&self) -> &str {
        "cd"
    }

    fn run(&self, args: &[String], context: &mut Context) -> Result {
        match args {
            [] => {
                let path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
                self.change_directory(path, context);
                Ok(Value::Empty)
            }
            [target] if target == "-" => match context.scope.get_env("OLDPWD") {
                Some(oldpwd) => {
                    let path = PathBuf::from(oldpwd);
                    self.change_directory(path, context);
                    let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
                    Ok(Value::String(path_to_string(&path)))
                }
                None => Err(ExecError::Message(String::from("cd: OLDPWD not set"))),
            },
            [target] => {
                let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
                path.push(target);
                path = path.canonicalize().unwrap_or(path);
                self.change_directory(path, context);
                Ok(Value::Empty)
            }
            _ => Err(ExecError::Value(Value::String(format!(
                "cd: invalid arguments: {}",
                args.join(" ")
            )))),
        }
    }
}

pub struct Pwd;

impl BuiltinCommand for Pwd {
    fn name(&self) -> &str {
        "pwd"
    }

    fn run(&self, _args: &[String], _context: &mut Context) -> Result {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        Ok(Value::String(path_to_string(&path)))
    }
}
