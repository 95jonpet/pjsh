use std::{collections::HashSet, process::Child, thread::JoinHandle};

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
    fn add_child_process(&mut self, child: std::process::Child) {
        self.child_processes.push(child);
    }

    fn add_thread(&mut self, thread: std::thread::JoinHandle<i32>) {
        self.threads.push(thread);
    }

    fn kill_all_processes(&mut self) {
        for mut child in std::mem::take(&mut self.child_processes) {
            let _ = child.kill(); // Results are safe to ignore.
        }
    }

    fn join_all_threads(&mut self) {
        for thread in std::mem::take(&mut self.threads) {
            let _ = thread.join(); // Results are safe to ignore.
        }
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
}
