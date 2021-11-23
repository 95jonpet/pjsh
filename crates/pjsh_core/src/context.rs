use std::sync::Arc;

use crate::{eval::scope::Scope, Host, StdHost};

pub struct Context {
    pub scope: Scope,
    pub arguments: Vec<String>,
    pub host: Arc<parking_lot::Mutex<dyn Host>>,
    pub last_exit: i32,
}

impl Default for Context {
    fn default() -> Self {
        let host = StdHost::default();
        let scope = Scope::default();

        for (key, value) in host.env_vars() {
            scope.set_env(
                key.to_string_lossy().to_string(),
                value.to_string_lossy().to_string(),
            );
        }

        if let Ok(current_exe) = std::env::current_exe() {
            scope.set_env(
                String::from("SHELL"),
                current_exe.to_string_lossy().to_string(),
            );
        }

        if let Ok(pwd) = std::env::current_dir() {
            scope.set_env(
                String::from("PWD"),
                pwd.to_string_lossy()
                    .trim_start_matches(r#"\\?\"#)
                    .to_string(),
            );
        }

        Self {
            scope,
            arguments: Vec::new(),
            host: Arc::new(parking_lot::Mutex::new(host)),
            last_exit: 0,
        }
    }
}
