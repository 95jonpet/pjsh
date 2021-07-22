mod io;

use crate::executor::exit_status::ExitStatus;

pub(crate) trait Builtin {
    fn execute(&self, args: &Vec<String>) -> ExitStatus;
}

pub(crate) fn builtin(program: &str) -> Option<Box<dyn Builtin>> {
    match program {
        "cd" => Some(Box::new(io::Cd::new())),
        _ => None,
    }
}
