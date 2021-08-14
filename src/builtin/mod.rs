pub(crate) mod io;
pub(crate) mod logic;

use crate::execution::{environment::Environment, exit_status::ExitStatus};

pub(crate) trait Builtin {
    fn execute(&self, args: &[String], env: &mut dyn Environment) -> ExitStatus;
}
