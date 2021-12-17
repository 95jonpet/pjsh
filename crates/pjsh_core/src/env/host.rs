use std::{
    collections::{HashMap, HashSet},
    ffi::{OsStr, OsString},
    process::Child,
};

/// A host is a shell's representation of its current environment.
///
/// The host is used to modify environment variables, and to keep track of child processes that a
/// shell spawns.
pub trait Host: Send {
    /// Prints a line of text to the host's stdout equivalent.
    fn println(&mut self, text: &str);

    /// Prints a line of text to the host's stderr equivalent.
    fn eprintln(&mut self, text: &str);

    /// Registers a child process in the host.
    ///
    /// The child process should originate from the shell, meaning that the shell should have
    /// spawned it.
    fn add_child_process(&mut self, child: Child);

    /// Return a list of all exited processes that have been spawend by the host, removing them from
    /// the list of tracked child processes.
    fn take_exited_child_processes(&mut self) -> HashSet<u32>;

    /// Returns all environment variables known by the host.
    fn env_vars(&self) -> HashMap<OsString, OsString>;

    /// Returns the value of an environment variable with a specific key.
    /// Returns `None` if the host cannot find the environment variable.
    fn get_env(&self, key: &OsStr) -> Option<OsString>;

    /// Sets the value for an environment variable with a specific key.
    /// Replaces any previously existing variable with the same key.
    fn set_env(&mut self, key: OsString, value: OsString);

    /// Removes an environment variable from the host.
    fn unset_env(&mut self, key: &OsStr);
}
