use std::{collections::HashMap, path::PathBuf};

pub(crate) trait Environment {
    fn find_program(&self, program: &str) -> Option<PathBuf>;
    fn var(&self, name: &str) -> Option<&String>;
    fn set_var(&mut self, name: String, value: String) -> Option<String>;
    fn unset_var(&mut self, name: &str) -> Option<String>;
}

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
        if let Some(path_env) = self.var("PATH") {
            // Define all possible file extensions that can be matched.
            let mut extensions = vec![""]; // Empty string = no file extension.
            if let Some(ext_env) = self.var("PATHEXT") {
                extensions.extend(ext_env.split(';'));
            }

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

    fn var(&self, name: &str) -> Option<&String> {
        self.vars.get(name)
    }

    fn set_var(&mut self, name: String, value: String) -> Option<String> {
        self.vars.insert(name, value)
    }

    fn unset_var(&mut self, name: &str) -> Option<String> {
        self.vars.remove(name)
    }
}
