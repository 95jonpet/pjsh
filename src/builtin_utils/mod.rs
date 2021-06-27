use crate::old::executor::Executor;

pub mod cd;

pub trait Builtin {
    fn execute(args: &Vec<String>, executor: &Executor) -> Result<i32, String>;
}
