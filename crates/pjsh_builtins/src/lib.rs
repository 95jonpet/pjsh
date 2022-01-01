mod alias;
mod echo;
mod env;
mod exit;
mod fs;
mod status;

pub fn builtin(name: &str) -> Option<Box<dyn pjsh_core::InternalCommand>> {
    match name {
        "alias" => Some(Box::new(alias::Alias {})),
        "cd" => Some(Box::new(fs::Cd {})),
        "drop" => Some(Box::new(env::Drop {})),
        "echo" => Some(Box::new(echo::Echo {})),
        "exit" => Some(Box::new(exit::Exit {})),
        "pwd" => Some(Box::new(fs::Pwd {})),
        "unalias" => Some(Box::new(alias::Unalias {})),
        _ => None,
    }
}
