use pjsh_core::{BuiltinCommand, Context, Result, Value};

pub struct Echo;

impl BuiltinCommand for Echo {
    fn name(&self) -> &str {
        "echo"
    }

    /// Prints space-separated arguments from `args` to stdout ending with a newline.
    fn run(&self, args: &[String], _context: &mut Context) -> Result {
        let mut output = String::new();
        let mut args = args.iter();
        if let Some(arg) = args.next() {
            output.push_str(arg);
        }

        for arg in args {
            output.push(' ');
            output.push_str(arg);
        }

        Ok(Value::String(output))
    }
}
