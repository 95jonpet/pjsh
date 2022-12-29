use std::path::PathBuf;

use clap::Parser;
use pjsh_core::{
    command::{Args, Command, CommandResult},
    utils::{path_to_string, word_var},
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

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match PwdOpts::try_parse_from(args.context.args()) {
            Ok(opts) => print_working_directory(opts, args),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Prints a contexts working directory to stdout.
fn print_working_directory(_opts: PwdOpts, args: &mut Args) -> CommandResult {
    let cwd = word_var(args.context, "PWD").map(PathBuf::from);

    if let Some(dir) = cwd {
        if let Err(error) = writeln!(args.io.stdout, "{}", path_to_string(dir)) {
            let _ = writeln!(args.io.stderr, "{NAME}: {error}");
            return CommandResult::code(status::GENERAL_ERROR);
        }

        return CommandResult::code(status::SUCCESS);
    }

    let _ = writeln!(args.io.stderr, "{NAME}: Unknown working directory.");
    CommandResult::code(status::GENERAL_ERROR)
}

#[cfg(test)]
mod tests {
    use pjsh_core::{Context, Value};

    use crate::utils::{file_contents, mock_io};

    use super::*;

    #[test]
    fn it_prints_the_current_working_directory() {
        let mut ctx = Context::default();
        let (mut io, mut stdout, mut stderr) = mock_io();

        ctx.set_var("PWD".into(), Value::Word("/current/path".into()));
        let pwd = Pwd {};

        let mut args = Args::new(&mut ctx, &mut io);
        if let CommandResult::Builtin(result) = pwd.run(&mut args) {
            assert_eq!(result.code, 0);
            assert!(result.actions.is_empty());
            assert_eq!(&file_contents(&mut stdout), "/current/path\n");
            assert_eq!(&file_contents(&mut stderr), "");
        } else {
            unreachable!()
        }
    }
}
