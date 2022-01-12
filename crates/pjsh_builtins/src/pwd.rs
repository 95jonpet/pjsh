use clap::Parser;
use pjsh_core::{
    command::Io,
    command::{Args, Command, CommandResult},
    utils::path_to_string,
    Context,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "pwd";

/// Print the shell's working directory.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct PwdOpts;

/// Implementation for the "pwd" built-in command.
#[derive(Clone)]
pub struct Pwd;
impl Command for Pwd {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, mut args: Args) -> CommandResult {
        match PwdOpts::try_parse_from(args.iter()) {
            Ok(opts) => print_working_directory(opts, &args.context, &mut args.io),
            Err(error) => utils::exit_with_parse_error(&mut args.io, error),
        }
    }
}

/// Prints a contexts working directory to stdout.
fn print_working_directory(_opts: PwdOpts, ctx: &Context, io: &mut Io) -> CommandResult {
    if let Some(dir) = ctx.scope.get_env("PWD") {
        if let Err(error) = writeln!(io.stdout, "{}", path_to_string(&dir)) {
            let _ = writeln!(io.stderr, "{NAME}: {error}");
            return CommandResult::code(status::GENERAL_ERROR);
        }

        return CommandResult::code(status::SUCCESS);
    }

    let _ = writeln!(io.stderr, "{NAME}: Unknown working directory.");
    CommandResult::code(status::GENERAL_ERROR)
}

#[cfg(test)]
mod tests {
    use pjsh_core::Context;

    use crate::utils::{file_contents, mock_io};

    use super::*;

    #[test]
    fn it_prints_the_current_working_directory() {
        let ctx = Context::new("test".into());
        let (io, mut stdout, mut stderr) = mock_io();

        ctx.scope.set_env("PWD".into(), "/current/path".into());
        let alias = Pwd {};

        let args = Args { context: ctx, io };
        let result = alias.run(args);

        assert_eq!(result.code, 0);
        assert!(result.actions.is_empty());
        assert_eq!(&file_contents(&mut stdout), "/current/path\n");
        assert_eq!(&file_contents(&mut stderr), "");
    }
}
