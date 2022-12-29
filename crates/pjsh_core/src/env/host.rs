use std::{collections::HashSet, process::Child, thread::JoinHandle};

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

    /// Registers a thread in the host.
    ///
    /// The thread should originate from the shell, meaning that the shell should have
    /// spawned it.
    fn add_thread(&mut self, thread: JoinHandle<i32>);

    /// Kills all registered child processes.
    fn kill_all_processes(&mut self);

    /// Returns the OS-assigned process identifier associated with this process.
    fn process_id(&self) -> u32;

    /// Joins all registered threads.
    fn join_all_threads(&mut self);

    /// Return a list of all exited processes that have been spawned by the host, removing them from
    /// the list of tracked child processes.
    fn take_exited_child_processes(&mut self) -> HashSet<u32>;
}
