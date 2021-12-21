use pjsh_core::{Context, InternalCommand, InternalIo};

pub struct Alias;

impl InternalCommand for Alias {
    fn name(&self) -> &str {
        "alias"
    }

    fn run(&self, args: &[String], context: &mut Context, io: &mut InternalIo) -> i32 {
        match args {
            [] => {
                for (key, value) in context.scope.aliases() {
                    let _ = writeln!(io.stdout, "alias {} = '{}'", &key, &value);
                }
                0
            }
            [key, op, value] if op == "=" => {
                context.scope.set_alias(key.clone(), value.clone());
                0
            }
            [_, op] if op == "=" => {
                let _ = writeln!(io.stderr, "alias: invalid arguments: {}", args.join(" "));
                2
            }
            keys => {
                let mut exit = 0;
                for key in keys {
                    if let Some(value) = context.scope.get_alias(key) {
                        let _ = writeln!(io.stdout, "alias {} = '{}'", &key, &value);
                    } else {
                        let _ = writeln!(io.stderr, "alias: {}: not found", &key);
                        exit = 1;
                    }
                }
                exit
            }
        }
    }
}

pub struct Unalias;

impl InternalCommand for Unalias {
    fn name(&self) -> &str {
        "unalias"
    }

    fn run(&self, args: &[String], context: &mut Context, io: &mut InternalIo) -> i32 {
        if args.is_empty() {
            let _ = writeln!(io.stderr, "unalias: usage: unalias name [name ...]");
            return 2;
        }

        for arg in args {
            context.scope.unset_alias(arg);
        }

        0
    }
}
