use pjsh_core::{BuiltinCommand, Context, ExecError, Result, Value};

pub struct Alias;

impl BuiltinCommand for Alias {
    fn name(&self) -> &str {
        "alias"
    }

    fn run(&self, args: &[String], context: &mut Context) -> Result {
        match args {
            [] => {
                let output = context
                    .scope
                    .aliases()
                    .into_iter()
                    .map(|(key, value)| format!("alias {} = '{}'", key, value))
                    .reduce(|a, b| a + "\n" + &b)
                    .unwrap_or_default(); // Empty when no aliases are set.
                Ok(Value::String(output))
            }
            [key] => {
                if let Some(value) = context.scope.get_alias(key) {
                    Ok(Value::String(format!("alias {} = '{}'", key, value)))
                } else {
                    Err(ExecError::Value(Value::String(format!(
                        "alias: {}: not found",
                        key
                    ))))
                }
            }
            [key, op, value] if op == "=" => {
                context.scope.set_alias(key.clone(), value.clone());
                Ok(Value::Empty)
            }
            _ => Err(ExecError::Value(Value::String(format!(
                "alias: invalid arguments: {}",
                args.join(" ")
            )))),
        }
    }
}

pub struct Unalias;

impl BuiltinCommand for Unalias {
    fn name(&self) -> &str {
        "unalias"
    }

    fn run(&self, args: &[String], context: &mut Context) -> Result {
        if args.is_empty() {
            return Err(ExecError::Value(Value::String(
                "unalias: usage: unalias name [name ...]".to_string(),
            )));
        }

        for arg in args {
            context.scope.unset_alias(arg);
        }

        Ok(Value::Empty)
    }
}
