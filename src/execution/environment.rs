use std::collections::HashMap;

pub(crate) struct ExecutionEnvironment {
    vars: HashMap<String, String>,
}

impl ExecutionEnvironment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn var(&self, name: &str) -> Option<&String> {
        self.vars.get(name)
    }

    pub fn set_var(&mut self, name: String, value: String) -> Option<String> {
        self.vars.insert(name, value)
    }

    pub fn unset_var(&mut self, name: &str) -> Option<String> {
        self.vars.remove(name)
    }
}
