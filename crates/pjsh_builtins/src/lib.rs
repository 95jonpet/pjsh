mod alias;
mod drop;
mod echo;
mod fs;

pub use alias::{Alias, Unalias};
pub use drop::Drop;
pub use echo::Echo;
pub use fs::{Cd, Pwd};

pub fn builtin(name: &str) -> Option<Box<dyn pjsh_core::InternalCommand>> {
    match name {
        "alias" => Some(Box::new(Alias {})),
        "cd" => Some(Box::new(Cd {})),
        "drop" => Some(Box::new(Drop {})),
        "echo" => Some(Box::new(Echo {})),
        "pwd" => Some(Box::new(Pwd {})),
        "unalias" => Some(Box::new(Unalias {})),
        _ => None,
    }
}
