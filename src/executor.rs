use crate::parser::SingleCommand;
use std::process::Command;

pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute_single(&self, mut single: SingleCommand) -> bool {
        match &single.cmd[..] {
            command => {
                let mut cmd = Command::new(command);
                cmd.args(&single.args);

                // if let Some(stdin) = single.stdin.borrow_mut().get_stdin() {
                //     cmd.stdin(stdin);
                // } else {
                //     return false;
                // }

                // if let Some(stdout) = single.stdout.borrow_mut().get_stdout() {
                //     cmd.stdout(stdout);
                // } else {
                //     return false;
                // }

                // if let Some(stderr) = single.stdin.borrow_mut().get_stderr() {
                //     cmd.stderr(stderr);
                // } else {
                //     return false;
                // }

                // if let Some(env) = single.env {
                //     cmd.envs(env);
                // }

                match cmd.status() {
                    Ok(child) => child.success(),
                    Err(e) => {
                        eprintln!("pjsh: {}: {}", single.cmd, e);
                        false
                    }
                }
            }
        }
    }
}
