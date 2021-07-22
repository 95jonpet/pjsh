mod io;
mod logic;

use crate::execution::exit_status::ExitStatus;

pub(crate) trait Builtin {
    fn execute(&self, args: &Vec<String>) -> ExitStatus;
}

pub(crate) fn builtin(program: &str) -> Option<Box<dyn Builtin>> {
    match program {
        "cd" => Some(Box::new(io::Cd {})),
        "exit" => Some(Box::new(io::Exit {})),
        "false" => Some(Box::new(logic::False {})),
        "true" => Some(Box::new(logic::True {})),
        _ => None,
    }
}
