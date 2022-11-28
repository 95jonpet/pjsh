use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
};

use pjsh_ast::Function;

use crate::{
    command::Command, file_descriptor::FileDescriptorError, FileDescriptor, Host, StdHost,
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
    pub fn get_var<'a>(&'a self, name: &str) -> Option<&'a str> {
        let variable_scopes = self
            .scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.vars.as_ref());
        for scope_vars in variable_scopes {
            if let Some(value) = scope_vars.get(name) {
                return Some(value);
            }
        }

        None
    }

    /// Returns all variable names within the current scope.
    pub fn get_var_names(&self) -> HashSet<String> {
        let variable_scopes = self
            .scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.vars.as_ref());
        let mut variables = HashSet::new();

        for scope_variables in variable_scopes {
            variables.extend(scope_variables.keys().cloned());
        }

        variables
    }

    /// Sets the value of a variable within the current scope.
    ///
    /// Parent scopes are not modified.
    pub fn set_var(&mut self, name: String, value: String) -> Option<String> {
        let scope_vars = self
            .scopes
            .iter_mut()
            .filter_map(|scope| scope.vars.as_mut())
            .last();
        if let Some(vars) = scope_vars {
            vars.insert(name, value)
        } else {
            unreachable!("A variable scope should always exist");
        }
    }

    /// Removes the value of a variable within the current scope. Returns the
    /// removed value.
    ///
    /// Parent scopes are not modified.
    ///
    /// TODO: Allow parent variables to be hidden when unset in a child scope.
    pub fn unset_var(&mut self, name: &str) -> Option<String> {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(vars) = &mut scope.vars {
                return vars.remove(name);
            }
        }

        None
    }

    /// Exports a variable from the shell's environment, causing the variable to be
    /// included in future program environments.
    ///
    /// The variable name must be known to the shell.
    pub fn export_var(&mut self, name: String) -> Result<(), String> {
        if self.get_var(&name).is_none() {
            return Err(format!("unknown variable: {name}"));
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
        self.scopes
            .iter()
            .flat_map(|scope| scope.exported_keys.iter())
            .map(|name| (name.as_str(), self.get_var(name).expect("defined variable")))
            .collect()
    }

    /// Returns a registered function with a specific name within the current scope.
    pub fn get_function<'a>(&'a self, name: &str) -> Option<&'a Function> {
        let function_scopes = self
            .scopes
            .iter()
            .rev()
            .filter_map(|scope| scope.functions.as_ref());
        for scope_functions in function_scopes {
            if let Some(value) = scope_functions.get(name) {
                return Some(value);
            }
        }

        None
    }

    /// Returns all registered function names within the current scope.
    pub fn get_function_names(&self) -> HashSet<String> {
        let function_scopes = self
            .scopes
            .iter()
            .filter_map(|scope| scope.functions.as_ref());
        let mut functions = HashSet::new();

        for scope_functions in function_scopes {
            functions.extend(scope_functions.keys().cloned());
        }

        functions
    }

    /// Registers a function within the current scope.
    pub fn register_function(&mut self, function: Function) {
        let scope_functions = self
            .scopes
            .iter_mut()
            .filter_map(|scope| scope.functions.as_mut())
            .last();
        if let Some(functions) = scope_functions {
            functions.insert(function.name.clone(), function);
        } else {
            unreachable!("A function scope should always exist");
        }
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
        self.scopes.last().map(|scope| scope.last_exit).unwrap_or(0)
    }

    pub fn register_exit(&mut self, exit: i32) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.last_exit = exit
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
                Some(HashMap::default()),
                Some(HashMap::default()),
                HashSet::default(),
                false,
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
    vars: Option<HashMap<String, String>>,

    /// A hash map containing functions that have been registered within this scope. More functions
    /// can be available through the [`Context`] itself.
    functions: Option<HashMap<String, Function>>,

    /// A hash set containing the names of all variables that this scope exports. More variables
    /// can be available through the [`Context`] itself.
    exported_keys: HashSet<String>,

    /// Determines whether or not user interaction is available within this scope.
    is_interactive: bool,

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
        vars: Option<HashMap<String, String>>,
        functions: Option<HashMap<String, Function>>,
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
            is_interactive: self.is_interactive,
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

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use super::*;

    #[test]
    fn is_interactive() {
        let interactive = || Scope {
            name: "interactive".to_owned(),
            args: None,
            vars: None,
            functions: None,
            exported_keys: HashSet::default(),
            is_interactive: true,
            last_exit: 0,
            file_descriptors: HashMap::default(),
            temporary_files: vec![],
        };
        let non_interactive = || Scope {
            name: "non-interactive".to_owned(),
            args: None,
            vars: None,
            functions: None,
            exported_keys: HashSet::default(),
            is_interactive: false,
            last_exit: 0,
            file_descriptors: HashMap::default(),
            temporary_files: vec![],
        };
        assert!(
            !Context::with_scopes(vec![]).is_interactive(),
            "non-interactive by default"
        );
        assert!(
            !Context::with_scopes(vec![interactive(), non_interactive()]).is_interactive(),
            "non-interactive if last scope is non-interactive"
        );
        assert!(
            Context::with_scopes(vec![non_interactive(), interactive()]).is_interactive(),
            "interactive if last scope is interactive"
        );
    }

    #[test]
    fn get_var() {
        let context = Context::with_scopes(vec![
            Scope {
                name: "outer".to_owned(),
                args: None,
                vars: Some(HashMap::from([
                    ("outer".to_owned(), "outer".to_owned()),
                    ("replace".to_owned(), "outer".to_owned()),
                ])),
                functions: None,
                exported_keys: HashSet::default(),
                is_interactive: false,
                last_exit: 0,
                file_descriptors: HashMap::default(),
                temporary_files: vec![],
            },
            Scope {
                name: "inner".to_owned(),
                args: None,
                vars: Some(HashMap::from([
                    ("inner".to_owned(), "inner".to_owned()),
                    ("replace".to_owned(), "inner".to_owned()),
                ])),
                functions: None,
                exported_keys: HashSet::default(),
                is_interactive: false,
                last_exit: 0,
                file_descriptors: HashMap::default(),
                temporary_files: vec![],
            },
        ]);

        assert_eq!(context.get_var("unset"), None);
        assert_eq!(context.get_var("outer"), Some("outer"));
        assert_eq!(context.get_var("inner"), Some("inner"));
        assert_eq!(context.get_var("replace"), Some("inner"));
    }

    #[test]
    fn it_replaces_its_args() {
        let new_args = vec!["replaced".to_owned(), "args".to_owned()];
        let mut context = Context::with_scopes(vec![Scope::new(
            "scope".into(),
            Some(vec!["original".to_owned(), "args".to_owned()]),
            None,
            None,
            HashSet::default(),
            false,
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
            None,
            None,
            HashSet::default(),
            false,
        )]);
        context.register_temporary_file(file.clone());

        context.pop_scope(); // The scope is dropped here.

        assert!(
            !file.exists(),
            "the file should be deleted when its owner scope is dropped"
        );
    }
}
