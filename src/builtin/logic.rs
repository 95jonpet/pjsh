use crate::execution::exit_status::ExitStatus;

use super::Builtin;

pub(super) struct False;
impl Builtin for False {
    fn execute(&self, _args: &Vec<String>) -> crate::execution::exit_status::ExitStatus {
        ExitStatus::new(1)
    }
}

pub(super) struct True;
impl Builtin for True {
    fn execute(&self, _args: &Vec<String>) -> crate::execution::exit_status::ExitStatus {
        ExitStatus::new(0)
    }
}
