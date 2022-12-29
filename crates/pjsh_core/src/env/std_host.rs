use std::{
    collections::HashSet,
    process::{self, Child},
    thread::JoinHandle,
};

use super::host::Host;

/// A host wrapping the Rust standard library.
#[derive(Default)]
pub struct StdHost {
    /// Child processes that the host has spawned.
    child_processes: Vec<Child>,
    /// Threads that the host has spawned.
    threads: Vec<JoinHandle<i32>>,
}

impl Host for StdHost {
    /// Prints a text line to stdout.
    fn println(&mut self, text: &str) {
        println!("{}", text);
    }

    /// Prints a text line to stderr.
    fn eprintln(&mut self, text: &str) {
        eprintln!("{}", text);
    }

    /// Marks a child process as owned by the host.
    fn add_child_process(&mut self, child: std::process::Child) {
        self.child_processes.push(child);
    }

    fn add_thread(&mut self, thread: std::thread::JoinHandle<i32>) {
        self.threads.push(thread);
    }

    fn kill_all_processes(&mut self) {
        for mut child in std::mem::take(&mut self.child_processes) {
            let _ = child.kill();
        }
    }

    fn process_id(&self) -> u32 {
        process::id()
    }

    fn join_all_threads(&mut self) {
        for thread in std::mem::take(&mut self.threads) {
            let _ = thread.join();
        }
    }

    /// Return a list of all exited processes that have been spawned by the host, removing them from
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
}
