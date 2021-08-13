use crate::execution::{environment::Environment, exit_status::ExitStatus};

use super::Builtin;

pub(crate) struct False;
impl Builtin for False {
    fn execute(
        &self,
        _args: &[String],
        _env: &mut impl Environment,
    ) -> crate::execution::exit_status::ExitStatus {
        ExitStatus::new(1)
    }
}

pub(crate) struct True;
impl Builtin for True {
    fn execute(
        &self,
        _args: &[String],
        _env: &mut impl Environment,
    ) -> crate::execution::exit_status::ExitStatus {
        ExitStatus::new(0)
    }
}
