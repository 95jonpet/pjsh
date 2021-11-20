use std::{
    collections::{HashMap, HashSet},
    ffi::{OsStr, OsString},
    process::Child,
};

pub trait Host: Send {
    fn println(&mut self, text: &str);
    fn eprintln(&mut self, text: &str);

    fn add_child_process(&mut self, child: Child);
    fn take_exited_child_processes(&mut self) -> HashSet<u32>;
    fn env_vars(&self) -> HashMap<OsString, OsString>;
    fn get_env(&self, key: &OsStr) -> Option<OsString>;
    fn set_env(&mut self, key: OsString, value: OsString);
    fn unset_env(&mut self, key: &OsStr);
}
