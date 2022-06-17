use std::{
    collections::{HashMap, HashSet},
    mem::replace,
    sync::Arc,
};

use pjsh_ast::Function;

use crate::{Host, StdHost};

/// An execution context consisting of a number of execution scopes.
#[derive(Clone)]
pub struct Context {
    /// Registered aliases keyed by their name.
    pub aliases: HashMap<String, String>,

    /// The context's host.
    pub host: Arc<parking_lot::Mutex<dyn Host>>,

    /// The exit code reported by the shell.
    pub last_exit: i32,

    /// Scopes in order of increasing specificity.
    scopes: Vec<Scope>,
}

impl Context {
    /// Returns `true` if the context is interactive.
    pub fn is_interactive(&self) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.is_interactive)
            .unwrap_or(false)
    }

    /// Returns the name of the current scope.
    pub fn name(&self) -> &str {
        self.scopes
            .last()
            .map(|scope| scope.name.as_str())
            .unwrap_or("global")
    }

    /// Constructs a new context from a set of scopes.
    ///
    /// Scopes should be provided in increasing order of specificity.
    pub fn with_scopes(scopes: Vec<Scope>) -> Self {
        Self {
            aliases: HashMap::default(),
            host: Arc::new(parking_lot::Mutex::new(StdHost::default())),
            last_exit: 0,
            scopes,
        }
    }

    /// Appends a scope to the context. This scope will become the innermost scope.
    pub fn push_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    /// Removes and returns the innermost scope in the context.
    ///
    /// # Panics
    ///
    /// Panics if there are no scopes within the context.
    pub fn pop_scope(&mut self) -> Scope {
        self.scopes.remove(self.scopes.len() - 1)
    }

    /// Returns the value of a variable within the current scope.
    pub fn get_var<'a>(&'a self, name: &str) -> Option<&'a str> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.vars.get(name) {
                return Some(value);
            }
        }

        None
    }

    /// Sets the value of a variable within the current scope.
    ///
    /// Parent scopes are not modfified.
    pub fn set_var(&mut self, name: String, value: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(name, value);
        }
    }

    /// Returns a collection with references to all exported variables within the current scope.
    pub fn exported_vars(&self) -> HashMap<&str, &str> {
        self.scopes
            .iter()
            .flat_map(|scope| scope.exported_keys.iter())
            .map(|name| (name.as_str(), self.get_var(name).expect("defined variable")))
            .collect()
    }

    /// Returns a registered function with a specific name within the current scope.
    pub fn get_function<'a>(&'a self, name: &str) -> Option<&'a Function> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.functions.get(name) {
                return Some(value);
            }
        }

        None
    }

    /// Registers a function within the current scope.
    pub fn register_function(&mut self, function: Function) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.functions.insert(function.name.clone(), function);
        }
    }

    /// Replaces the positional arguments for the current scope and returns its old value.
    pub fn replace_args(&mut self, args: Vec<String>) -> Vec<String> {
        if let Some(scope) = self.scopes.last_mut() {
            replace(&mut scope.args, args)
        } else {
            Vec::new()
        }
    }

    /// Returns a slice containing all positional arguments within the current scope.
    pub fn args(&self) -> &[String] {
        self.scopes
            .last()
            .map(|scope| scope.args.as_slice())
            .unwrap_or_default()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            aliases: Default::default(),
            host: Arc::new(parking_lot::Mutex::new(StdHost::default())),
            last_exit: Default::default(),
            scopes: vec![Scope::new(
                "global".to_owned(),
                Vec::default(),
                HashMap::default(),
                HashMap::default(),
                HashSet::default(),
                false,
            )],
        }
    }
}

/// An execution scope containing variables and functions.
///
/// A scope only contains values added within its reference. In reality, scopes are nested within a
/// wider [`Context`], transitively providing access to values within parent scopes.
#[derive(Clone)]
pub struct Scope {
    /// A name used to identify the scope.
    name: String,

    /// Positional arguments within the scope.
    args: Vec<String>,

    /// A hash map containing variables that have been registered within this scope. More variables
    /// can be available through the [`Context`] itself.
    vars: HashMap<String, String>,

    /// A hash map containing functions that have been registered within this scope. More functions
    /// can be available through the [`Context`] itself.
    functions: HashMap<String, Function>,

    /// A hash set containing the names of all variables that this scope exports. More variables
    /// can be available through the [`Context`] itself.
    exported_keys: HashSet<String>,

    /// Determines whether or not user interaction is available within this scope.
    is_interactive: bool,
}

impl Scope {
    /// Constructs a new scope.
    pub fn new(
        name: String,
        args: Vec<String>,
        vars: HashMap<String, String>,
        functions: HashMap<String, Function>,
        exported_keys: HashSet<String>,
        is_interactive: bool,
    ) -> Self {
        Self {
            name,
            args,
            vars,
            functions,
            exported_keys,
            is_interactive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_interactive() {
        let interactive = || Scope {
            name: "interactive".to_owned(),
            args: vec![],
            vars: HashMap::default(),
            functions: HashMap::default(),
            exported_keys: HashSet::default(),
            is_interactive: true,
        };
        let non_interacive = || Scope {
            name: "non-interactive".to_owned(),
            args: vec![],
            vars: HashMap::default(),
            functions: HashMap::default(),
            exported_keys: HashSet::default(),
            is_interactive: false,
        };
        assert!(
            !Context::with_scopes(vec![]).is_interactive(),
            "non-interactive by default"
        );
        assert!(
            !Context::with_scopes(vec![interactive(), non_interacive()]).is_interactive(),
            "non-interactive if last scope is non-interactive"
        );
        assert!(
            Context::with_scopes(vec![non_interacive(), interactive()]).is_interactive(),
            "interactive if last scope is interactive"
        );
    }

    #[test]
    fn get_var() {
        let context = Context::with_scopes(vec![
            Scope {
                name: "outer".to_owned(),
                args: vec![],
                vars: HashMap::from([
                    ("outer".to_owned(), "outer".to_owned()),
                    ("replace".to_owned(), "outer".to_owned()),
                ]),
                functions: HashMap::default(),
                exported_keys: HashSet::default(),
                is_interactive: false,
            },
            Scope {
                name: "inner".to_owned(),
                args: vec![],
                vars: HashMap::from([
                    ("inner".to_owned(), "inner".to_owned()),
                    ("replace".to_owned(), "inner".to_owned()),
                ]),
                functions: HashMap::default(),
                exported_keys: HashSet::default(),
                is_interactive: false,
            },
        ]);

        assert_eq!(context.get_var("unset"), None);
        assert_eq!(context.get_var("outer"), Some("outer"));
        assert_eq!(context.get_var("inner"), Some("inner"));
        assert_eq!(context.get_var("replace"), Some("inner"));
    }
}
