use pjsh_core::{BuiltinCommand, Context, ExecError, Result, Value};

pub struct Drop;

impl BuiltinCommand for Drop {
    fn name(&self) -> &str {
        "drop"
    }

    /// Drops all environment variables with keys defined in `args`.
    fn run(&self, args: &[String], context: &mut Context) -> Result {
        if args.is_empty() {
            return Err(ExecError::Value(Value::String(
                "drop: missing keys to drop.".to_string(),
            )));
        }

        for arg in args {
            context.scope.unset_env(arg);
        }

        Ok(Value::Empty)
    }
}
