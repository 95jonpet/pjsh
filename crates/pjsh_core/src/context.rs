use std::sync::Arc;

use crate::{eval::scope::Scope, Host, StdHost};

/// Status code representing success.
///
/// This is the only valid status indicating the successful completion of a program.
/// Thus, all other status codes represent various error states.
const EXIT_SUCCESS: i32 = 0;

#[derive(Clone)]
pub struct Context {
    pub name: String,
    pub scope: Scope,
    pub arguments: Vec<String>,
    pub host: Arc<parking_lot::Mutex<dyn Host>>,
    pub last_exit: i32,
}

impl Context {
    pub fn new(name: String) -> Self {
        let host = StdHost::default();
        let scope = Scope::default();
        let args = vec![name.clone()];

        Self {
            name,
            scope,
            arguments: args,
            host: Arc::new(parking_lot::Mutex::new(host)),
            last_exit: EXIT_SUCCESS,
        }
    }

    pub fn fork(&self, name: String) -> Self {
        let args = vec![name.clone()];
        Self {
            name,
            scope: self.scope.fork(),
            arguments: args,
            host: Arc::clone(&self.host),
            last_exit: self.last_exit,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new(String::from("pjsh"))
    }
}
