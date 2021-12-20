use pjsh_core::{Context, InternalCommand};

pub struct Echo;

impl InternalCommand for Echo {
    fn name(&self) -> &str {
        "echo"
    }

    fn run(&self, args: &[String], _: &mut Context, io: &mut pjsh_core::InternalIo) -> i32 {
        let mut output = String::new();
        let mut args = args.iter();
        if let Some(arg) = args.next() {
            output.push_str(arg);
        }

        for arg in args {
            output.push(' ');
            output.push_str(arg);
        }

        // Use exit code to signal success. If stdout cannot be written to, stderr is probably not
        // going to work either.
        match writeln!(io.stdout, "{}", &output) {
            Ok(_) => 0,
            Err(error) => {
                let _ = writeln!(io.stderr, "echo: {}", error);
                1
            }
        }
    }
}
