use std::{
    collections::{HashMap, HashSet},
    process::Child,
};

use super::host::Host;

pub struct StdHost {
    child_processes: Vec<Child>,
}

impl Host for StdHost {
    fn println(&mut self, text: &str) {
        println!("{}", text)
    }

    fn eprintln(&mut self, text: &str) {
        eprintln!("{}", text)
    }

    fn add_child_process(&mut self, child: std::process::Child) {
        self.child_processes.push(child)
    }

    fn take_exited_child_processes(&mut self) -> HashSet<u32> {
        let mut exited = HashSet::new();
        for child in &mut self.child_processes {
            if !matches!(child.try_wait(), Ok(None)) {
                exited.insert(child.id());
                let _ = child.wait(); // Ensure that stdin is dropped.
            }
        }

        // Remove exited processes from the internal data structure.
        self.child_processes
            .retain(|child| !exited.contains(&child.id()));

        exited
    }

    fn env_vars(&self) -> HashMap<std::ffi::OsString, std::ffi::OsString> {
        std::env::vars_os().collect::<HashMap<_, _>>()
    }

    fn get_env(&self, key: &std::ffi::OsStr) -> Option<std::ffi::OsString> {
        std::env::var_os(key)
    }

    fn set_env(&mut self, key: std::ffi::OsString, value: std::ffi::OsString) {
        std::env::set_var(key, value)
    }

    fn unset_env(&mut self, key: &std::ffi::OsStr) {
        std::env::remove_var(key)
    }
}

impl Default for StdHost {
    fn default() -> Self {
        Self {
            child_processes: Vec::new(),
        }
    }
}
