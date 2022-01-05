mod alias;
mod echo;
mod env;
mod exit;
mod fs;
mod logic;
pub mod status;

/// Returns a built-in [`InternalCommand`] with a given `name`.
pub fn builtin(name: &str) -> Option<Box<dyn pjsh_core::InternalCommand>> {
    match name {
        "alias" => Some(Box::new(alias::Alias {})),
        "cd" => Some(Box::new(fs::Cd {})),
        "unset" => Some(Box::new(env::Unset {})),
        "echo" => Some(Box::new(echo::Echo {})),
        "exit" => Some(Box::new(exit::Exit {})),
        "false" => Some(Box::new(logic::False {})),
        "pwd" => Some(Box::new(fs::Pwd {})),
        "true" => Some(Box::new(logic::True {})),
        "unalias" => Some(Box::new(alias::Unalias {})),
        _ => None,
    }
}
