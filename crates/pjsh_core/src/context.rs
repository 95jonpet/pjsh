use std::sync::Arc;

use crate::{eval::scope::Scope, Host, StdHost};

pub struct Context {
    pub scope: Scope,
    pub arguments: Vec<String>,
    pub host: Arc<parking_lot::Mutex<dyn Host>>,
    pub last_exit: i32,
}

impl Context {
    pub fn new() -> Self {
        let host = StdHost::default();
        let scope = Scope::default();

        Self {
            scope,
            arguments: Vec::new(),
            host: Arc::new(parking_lot::Mutex::new(host)),
            last_exit: 0, // Success
        }
    }

    pub fn fork(&self) -> Self {
        Self {
            scope: self.scope.fork(),
            arguments: Vec::new(),
            host: Arc::clone(&self.host),
            last_exit: self.last_exit,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
