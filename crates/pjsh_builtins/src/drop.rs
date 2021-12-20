use pjsh_core::{Context, InternalCommand};

pub struct Drop;

impl InternalCommand for Drop {
    fn name(&self) -> &str {
        "drop"
    }

    /// Drops all environment variables with keys defined in `args`.
    fn run(&self, args: &[String], context: &mut Context, io: &mut pjsh_core::InternalIo) -> i32 {
        if args.is_empty() {
            let _ = writeln!(io.stderr, "drop: missing keys to drop");
            return 2;
        }

        for arg in args {
            context.scope.unset_env(arg);
        }

        0
    }
}
