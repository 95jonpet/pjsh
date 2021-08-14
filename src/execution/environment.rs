use std::{collections::HashMap, env, path::PathBuf};

/// An execution environment.
pub(crate) trait Environment {
    /// Finds the path for a program.
    fn find_program(&self, program: &str) -> Option<PathBuf>;
    /// Returns the value of a named variable.
    fn var(&self, name: &str) -> Option<String>;
    /// Sets the value of a named variable.
    fn set_var(&mut self, name: String, value: String) -> Option<String>;
    /// Clears the value of a named variable.
    fn unset_var(&mut self, name: &str) -> Option<String>;
}

/// An execution environment suitable for Unix systems.
pub(crate) struct UnixEnvironment {
    vars: HashMap<String, String>,
}

impl Default for UnixEnvironment {
    fn default() -> Self {
        Self {
            vars: std::env::vars().collect(),
        }
    }
}

impl Environment for UnixEnvironment {
    fn find_program(&self, program: &str) -> Option<PathBuf> {
        if let Some(path_env) = self.var("PATH") {
            // Define all possible paths using paths in PATH.
            let possible_paths = path_env
                .split(';')
                .map(|directory| {
                    [directory, &(program.to_string())]
                        .iter()
                        .collect::<PathBuf>()
                        .canonicalize()
                })
                .filter(Result::is_ok)
                .map(Result::unwrap);

            for path in possible_paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    fn var(&self, name: &str) -> Option<String> {
        self.vars.get(name).map(String::to_owned)
    }

    fn set_var(&mut self, name: String, value: String) -> Option<String> {
        self.vars.insert(name, value)
    }

    fn unset_var(&mut self, name: &str) -> Option<String> {
        self.vars.remove(name)
    }
}

/// An execution environment suitable for Windows systems.
pub(crate) struct WindowsEnvironment {
    vars: HashMap<String, String>,
}

impl Default for WindowsEnvironment {
    fn default() -> Self {
        Self {
            vars: std::env::vars().collect(),
        }
    }
}

impl Environment for WindowsEnvironment {
    fn find_program(&self, program: &str) -> Option<PathBuf> {
        // Define all possible file extensions that can be matched.
        let mut extensions = vec![String::new()]; // Empty string = no file extension.
        if let Some(ext_env) = self.var("PATHEXT") {
            extensions.extend(ext_env.split(';').map(str::to_owned));
        }

        if let Some(path_env) = self.var("Path").or_else(|| env::var("Path").ok()) {
            // Define all possible paths using paths in PATH combined with all possible extensions.
            let possible_paths = path_env.split(';').flat_map(|directory| {
                extensions
                    .iter()
                    .map(move |extension| {
                        [directory, &(program.to_string() + extension)]
                            .iter()
                            .collect::<PathBuf>()
                            .canonicalize()
                    })
                    .filter(Result::is_ok)
                    .map(Result::unwrap)
            });

            for path in possible_paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    fn var(&self, name: &str) -> Option<String> {
        self.vars.get(name).map(String::to_owned)
    }

    fn set_var(&mut self, name: String, value: String) -> Option<String> {
        self.vars.insert(name, value)
    }

    fn unset_var(&mut self, name: &str) -> Option<String> {
        self.vars.remove(name)
    }
}
