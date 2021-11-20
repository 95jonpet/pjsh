mod alias;
mod drop;
mod echo;
mod fs;

pub use alias::{Alias, Unalias};
pub use drop::Drop;
pub use echo::Echo;
pub use fs::{Cd, Pwd};

pub fn all_builtins() -> Vec<Box<dyn pjsh_core::BuiltinCommand>> {
    vec![
        Box::new(Alias {}),
        Box::new(Cd {}),
        Box::new(Drop {}),
        Box::new(Echo {}),
        Box::new(Pwd {}),
        Box::new(Unalias {}),
    ]
}
