use std::{
    collections::{HashMap, HashSet},
    process::Child,
};

use super::host::Host;

/// A host wrapping the Rust standard library.
#[derive(Default)]
pub struct StdHost {
    /// Child processes that the host has spawned.
    child_processes: Vec<Child>,
}

impl Host for StdHost {
    /// Prints a text line to stdout.
    fn println(&mut self, text: &str) {
        println!("{}", text)
    }

    /// Prints a text line to stderr.
    fn eprintln(&mut self, text: &str) {
        eprintln!("{}", text)
    }

    /// Marks a child process as owned by the host.
    fn add_child_process(&mut self, child: std::process::Child) {
        self.child_processes.push(child)
    }

    /// Return a list of all exited processes that have been spawend by the host, removing them from
    /// the list of tracked child processes.
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

    /// Returns all environment variables known by the host.
    fn env_vars(&self) -> HashMap<std::ffi::OsString, std::ffi::OsString> {
        std::env::vars_os().collect::<HashMap<_, _>>()
    }

    /// Returns the value of an environment variable with a specific key.
    /// Returns `None` if the host cannot find the environment variable.
    fn get_env(&self, key: &std::ffi::OsStr) -> Option<std::ffi::OsString> {
        std::env::var_os(key)
    }

    /// Sets the value for an environment variable with a specific key.
    /// Replaces any previously existing variable with the same key.
    fn set_env(&mut self, key: std::ffi::OsString, value: std::ffi::OsString) {
        std::env::set_var(key, value)
    }

    /// Removes an environment variable from the host.
    fn unset_env(&mut self, key: &std::ffi::OsStr) {
        std::env::remove_var(key)
    }
}
