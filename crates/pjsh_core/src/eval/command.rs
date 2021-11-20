use crate::{Context, Result};

pub trait BuiltinCommand: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, args: &[String], context: &mut Context) -> Result;
}
