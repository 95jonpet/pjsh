use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
};

use pjsh_ast::Function;

use crate::{
    command::Command, file_descriptor::FileDescriptorError, utils::word_var, FileDescriptor, Host,
    StdHost,
};

/// An execution context consisting of a number of execution scopes.
pub struct Context {
    /// Registered aliases keyed by their name.
    pub aliases: HashMap<String, String>,

    /// The context's host.
    pub host: Arc<parking_lot::Mutex<dyn Host>>,

    /// Scopes in order of increasing specificity.
    scopes: Vec<Scope>,

    /// Built-in commands in the context.
    pub builtins: HashMap<String, Box<dyn Command>>,
}

impl Context {
    /// Clones a scope.
    pub fn try_clone(&self) -> std::io::Result<Self> {
        let mut scopes = Vec::with_capacity(self.scopes.len());
        for scope in &self.scopes {
            scopes.push(scope.try_clone()?);
        }

        Ok(Self {
            aliases: self.aliases.clone(),
            host: Arc::clone(&self.host),
            scopes,
            builtins: self.builtins.clone(),
        })
    }

    /// Returns the name of the current scope.
    pub fn name(&self) -> &str {
        self.scopes
            .last()
            .map_or("global", |scope| scope.name.as_str())
    }

    /// Constructs a new context from a set of scopes.
    ///
    /// Scopes should be provided in increasing order of specificity.
    pub fn with_scopes(scopes: Vec<Scope>) -> Self {
        Self {
            aliases: HashMap::default(),
            host: Arc::new(parking_lot::Mutex::new(StdHost::default())),
            scopes,
            builtins: HashMap::new(),
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
    pub fn get_var<'a>(&'a self, name: &str) -> Option<&'a Value> {
        let Some(Some(value)) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.vars.get(name)) else {
                return None;
            };

        Some(value)
    }

    /// Returns all variable names within the current scope.
    pub fn get_var_names(&self) -> HashSet<String> {
        let mut variables = HashSet::new();

        for scope in &self.scopes {
            variables.extend(scope.vars.keys().cloned());
        }

        variables
    }

    /// Sets the value of a variable within the current scope.
    ///
    /// Parent scopes are not modified.
    pub fn set_var(&mut self, name: String, value: Value) -> Option<Value> {
        let Some(scope) = self.scopes.last_mut() else {
            return None;
        };

        scope.vars.insert(name, Some(value)).flatten()
    }

    /// Removes the value of a variable within the current scope. Returns the
    /// removed value.
    ///
    /// Parent scopes are not modified.
    pub fn unset_var(&mut self, name: &str) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        // Remove the function if it is defined in the current scope.
        if scope.vars.remove(name).is_some() {
            return;
        }

        // Shadow the function if declared in a parent scope.
        scope.vars.insert(name.to_owned(), None);
    }

    /// Exports a variable from the shell's environment, causing the variable to be
    /// included in future program environments.
    ///
    /// The variable name must be known to the shell.
    pub fn export_var(&mut self, name: String) -> Result<(), String> {
        match self.get_var(&name) {
            None => return Err(format!("unknown variable: {name}")),
            Some(Value::List(_)) => return Err(format!("lists are not exportable: {name}")),
            _ => (),
        }

        self.scopes
            .last_mut()
            .expect("scope exists") // A scope should always exist here.
            .exported_keys
            .insert(name);

        Ok(())
    }

    /// Returns a collection with references to all exported variables within the current scope.
    pub fn exported_vars(&self) -> HashMap<&str, &str> {
        let keys = self
            .scopes
            .iter()
            .flat_map(|scope| scope.exported_keys.iter());

        keys.map(|key| {
            (
                key.as_str(),
                word_var(self, key).expect("only word variables should be exportable"),
            )
        })
        .collect()
    }

    /// Returns a registered function with a specific name within the current scope.
    pub fn get_function<'a>(&'a self, name: &str) -> Option<&'a Function> {
        let Some(Some(function)) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.functions.get(name)) else {
                return None;
            };

        Some(function)
    }

    /// Returns all registered function names within the current scope.
    pub fn get_function_names(&self) -> HashSet<String> {
        let scopes = self.scopes.iter();
        let mut functions = HashSet::new();

        for scope in scopes {
            functions.extend(scope.functions.keys().cloned());
        }

        functions
    }

    /// Registers a function within the current scope.
    pub fn register_function(&mut self, function: Function) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        let name = function.name.clone();
        scope.functions.insert(name, Some(function));
    }

    /// Unregisters a function within the current scope.
    pub fn unregister_function(&mut self, name: &str) {
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };

        // Remove the function if it is defined in the current scope.
        if scope.functions.remove(name).is_some() {
            return;
        }

        // Shadow the function if declared in a parent scope.
        scope.functions.insert(name.to_owned(), None);
    }

    /// Returns a built-in command matching a name.
    pub fn get_builtin(&self, name: &str) -> Option<&dyn Command> {
        self.builtins.get(name).map(|builtin| builtin.as_ref())
    }

    /// Registers a built-in command within the scope.
    pub fn register_builtin(&mut self, builtin: Box<dyn Command>) {
        self.builtins.insert(builtin.name().to_owned(), builtin);
    }

    /// Registers a temporary file within the current scope.
    pub fn register_temporary_file(&mut self, path: PathBuf) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.temporary_files.push(path);
        }
    }

    /// Replaces the positional arguments for the current scope and returns its old value.
    pub fn replace_args(&mut self, args: Option<Vec<String>>) -> Option<Vec<String>> {
        let scope = self.scopes.last_mut().expect("a scope exists");
        std::mem::replace(&mut scope.args, args)
    }

    /// Returns a slice containing all positional arguments within the current scope.
    pub fn args(&self) -> &[String] {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.args.as_deref())
            .unwrap_or_default()
    }

    /// Returns the last exit code reported by the shell.
    pub fn last_exit(&self) -> i32 {
        self.scopes.last().map_or(0, |scope| scope.last_exit)
    }

    pub fn register_exit(&mut self, exit: i32) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.last_exit = exit;
        }
    }

    pub fn get_file_descriptor(&self, index: usize) -> Option<&FileDescriptor> {
        for scope in self.scopes.iter().rev() {
            if let Some(file_descriptor) = scope.file_descriptors.get(&index) {
                return Some(file_descriptor);
            }
        }
        None
    }

    pub fn set_file_descriptor(&mut self, index: usize, file_descriptor: FileDescriptor) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.file_descriptors.insert(index, file_descriptor);
        }
    }

    pub fn input(&mut self, index: usize) -> Option<Result<Stdio, FileDescriptorError>> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(file_descriptor) = scope.file_descriptors.get_mut(&index) {
                return Some(file_descriptor.input());
            }
        }
        None
    }

    pub fn output(&mut self, index: usize) -> Option<Result<Stdio, FileDescriptorError>> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(file_descriptor) = scope.file_descriptors.get_mut(&index) {
                return Some(file_descriptor.output());
            }
        }
        None
    }

    pub fn reader(
        &mut self,
        index: usize,
    ) -> Option<Result<Box<dyn Read + Send>, FileDescriptorError>> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(file_descriptor) = scope.file_descriptors.get_mut(&index) {
                return Some(file_descriptor.reader());
            }
        }
        None
    }

    pub fn writer(
        &mut self,
        index: usize,
    ) -> Option<Result<Box<dyn Write + Send>, FileDescriptorError>> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(file_descriptor) = scope.file_descriptors.get_mut(&index) {
                return Some(file_descriptor.writer());
            }
        }
        None
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            aliases: Default::default(),
            host: Arc::new(parking_lot::Mutex::new(StdHost::default())),
            scopes: vec![Scope::new(
                "global".to_owned(),
                Some(Vec::default()),
                HashMap::default(),
                HashMap::default(),
                HashSet::default(),
            )],
            builtins: Default::default(),
        }
    }
}

/// An execution scope containing variables and functions.
///
/// A scope only contains values added within its reference. In reality, scopes are nested within a
/// wider [`Context`], transitively providing access to values within parent scopes.
pub struct Scope {
    /// A name used to identify the scope.
    name: String,

    /// Positional arguments within the scope.
    args: Option<Vec<String>>,

    /// A hash map containing variables that have been registered within this scope. More variables
    /// can be available through the [`Context`] itself.
    vars: HashMap<String, Option<Value>>,

    /// A hash map containing functions that have been registered within this scope. More functions
    /// can be available through the [`Context`] itself.
    functions: HashMap<String, Option<Function>>,

    /// A hash set containing the names of all variables that this scope exports. More variables
    /// can be available through the [`Context`] itself.
    exported_keys: HashSet<String>,

    /// The exit code reported by the shell.
    last_exit: i32,

    /// File descriptors.
    file_descriptors: HashMap<usize, FileDescriptor>,

    /// Temporary files owned by the scope.
    temporary_files: Vec<PathBuf>,
}

impl Scope {
    /// Constructs a new scope.
    pub fn new(
        name: String,
        args: Option<Vec<String>>,
        vars: HashMap<String, Option<Value>>,
        functions: HashMap<String, Option<Function>>,
        exported_keys: HashSet<String>,
    ) -> Self {
        Self {
            name,
            args,
            vars,
            functions,
            exported_keys,
            last_exit: 0,
            file_descriptors: Default::default(),
            temporary_files: Vec::new(),
        }
    }

    /// Clones a scope.
    pub fn try_clone(&self) -> std::io::Result<Self> {
        let mut file_descriptors = HashMap::with_capacity(self.file_descriptors.len());
        for (key, value) in &self.file_descriptors {
            file_descriptors.insert(*key, value.try_clone()?);
        }

        Ok(Self {
            name: self.name.clone(),
            args: self.args.clone(),
            vars: self.vars.clone(),
            functions: self.functions.clone(),
            exported_keys: self.exported_keys.clone(),
            last_exit: self.last_exit,
            file_descriptors,
            temporary_files: self.temporary_files.clone(),
        })
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        // Remove all temporary files registered within the scope.
        for path in std::mem::take(&mut self.temporary_files) {
            if let Err(error) = std::fs::remove_file(path) {
                eprintln!("{error}");
            }
        }
    }
}

/// A single value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A value consisting of a single word.
    Word(String),

    /// A value consisting of 0 or more words.
    List(Vec<String>),
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use pjsh_ast::Block;

    use super::*;

    #[test]
    fn get_var() {
        let context = Context::with_scopes(vec![
            Scope {
                name: "outer".to_owned(),
                args: None,
                vars: HashMap::from([
                    ("outer".to_owned(), Some(Value::Word("outer".to_owned()))),
                    ("both".to_owned(), Some(Value::Word("outer".to_owned()))),
                ]),
                functions: HashMap::default(),
                exported_keys: HashSet::default(),
                last_exit: 0,
                file_descriptors: HashMap::default(),
                temporary_files: vec![],
            },
            Scope {
                name: "inner".to_owned(),
                args: None,
                vars: HashMap::from([
                    ("inner".to_owned(), Some(Value::Word("inner".to_owned()))),
                    ("both".to_owned(), Some(Value::Word("inner".to_owned()))),
                ]),
                functions: HashMap::default(),
                exported_keys: HashSet::default(),
                last_exit: 0,
                file_descriptors: HashMap::default(),
                temporary_files: vec![],
            },
        ]);

        assert_eq!(context.get_var("unset"), None);
        assert_eq!(context.get_var("outer"), Some(&Value::Word("outer".into())));
        assert_eq!(context.get_var("inner"), Some(&Value::Word("inner".into())));
        assert_eq!(context.get_var("both"), Some(&Value::Word("inner".into())));
    }

    #[test]
    fn it_replaces_its_args() {
        let new_args = vec!["replaced".to_owned(), "args".to_owned()];
        let mut context = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            Some(vec!["original".to_owned(), "args".to_owned()]),
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);

        context.replace_args(Some(new_args.clone()));

        assert_eq!(context.args(), &new_args[..]);
    }

    #[test]
    fn it_deletes_temporary_files_when_their_scope_is_dropped() {
        let mut file = temp_dir();
        file.push("scope-file");
        std::fs::write(&file, "file contents").expect("file is writable");
        let mut context = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            None,
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
        )]);
        context.register_temporary_file(file.clone());

        context.pop_scope(); // The scope is dropped here.

        assert!(
            !file.exists(),
            "the file should be deleted when its owner scope is dropped"
        );
    }

    #[test]
    fn it_unregisters_functions() {
        let outer_fn = Function {
            name: "outer".into(),
            args: Vec::default(),
            list_arg: None,
            body: Block::default(),
        };
        let inner_fn = Function {
            name: "inner".into(),
            args: Vec::default(),
            list_arg: None,
            body: Block::default(),
        };

        let mut context = Context::with_scopes(vec![
            Scope::new(
                "outer".into(),
                None,
                HashMap::default(),
                HashMap::from([("outer".to_string(), Some(outer_fn.clone()))]),
                HashSet::default(),
            ),
            Scope::new(
                "inner".into(),
                None,
                HashMap::default(),
                HashMap::from([("inner".to_string(), Some(inner_fn.clone()))]),
                HashSet::default(),
            ),
        ]);

        context.unregister_function("outer");
        context.unregister_function("inner");

        assert_eq!(context.get_function("outer"), None);
        assert_eq!(context.get_function("inner"), None);

        context.pop_scope();
        assert_eq!(
            context.get_function("outer"),
            Some(&outer_fn),
            "the function should not be dropped from the outer scope"
        );
    }

    #[test]
    fn it_unsets_vars() {
        let mut context = Context::with_scopes(vec![
            Scope::new(
                "outer".into(),
                None,
                HashMap::from([("outer".to_string(), Some(Value::Word("outer".into())))]),
                HashMap::default(),
                HashSet::default(),
            ),
            Scope::new(
                "inner".into(),
                None,
                HashMap::from([("inner".to_string(), Some(Value::Word("inner".into())))]),
                HashMap::default(),
                HashSet::default(),
            ),
        ]);

        context.unset_var("outer");
        context.unset_var("inner");

        assert_eq!(context.get_var("outer"), None);
        assert_eq!(context.get_var("inner"), None);

        context.pop_scope();
        assert_eq!(
            context.get_var("outer"),
            Some(&Value::Word("outer".into())),
            "the var should not be dropped from the outer scope"
        );
    }
}
