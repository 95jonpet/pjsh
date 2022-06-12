use pjsh_core::command::{Command, CommandResult};

use crate::Executor;

/// Constructs a test executor with some built-in test commands.
pub(crate) fn test_executor() -> Executor {
    Executor::new(vec![
        Box::new(EchoTestCommand {}),
        Box::new(ExitTestCommand {}),
    ])
}

/// Test version of the "echo" command.
///
/// Prints
#[derive(Clone)]
struct EchoTestCommand;
impl Command for EchoTestCommand {
    fn name(&self) -> &str {
        "echo"
    }

    fn run(&self, mut args: pjsh_core::command::Args) -> pjsh_core::command::CommandResult {
        let _ = writeln!(
            args.io.stdout,
            "{}",
            &args.context.lock().args()[1..].join(" ")
        );
        CommandResult::code(0)
    }
}

/// Test version of the "exit" command.
///
/// Exits with a specified code.
#[derive(Clone)]
struct ExitTestCommand;
impl Command for ExitTestCommand {
    fn name(&self) -> &str {
        "exit"
    }

    fn run(&self, args: pjsh_core::command::Args) -> pjsh_core::command::CommandResult {
        match args.context.lock().args().get(1).unwrap().parse::<i32>() {
            Ok(code) => CommandResult::code(code),
            Err(_) => unreachable!("only used correctly during tests"),
        }
    }
}
