use clap::Parser;
use pjsh_core::{
    command::{Args, Io},
    command::{Command, CommandResult},
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "echo";

/// Print a line of text.
///
/// This is a built-in shell command.
#[derive(Debug, Parser)]
#[clap(name = NAME, version)]
struct EchoOpts {
    /// Do not print trailing newline.
    #[clap(short, long)]
    no_newline: bool,

    /// Text strings to print.
    text: Vec<String>,
}

/// Implementation for the "echo" built-in command.
#[derive(Clone)]
pub struct Echo;
impl Command for Echo {
    fn name(&self) -> &str {
        NAME
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match EchoOpts::try_parse_from(args.context.args()) {
            Ok(opts) => print_text(opts, args.io),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Prints text to stdout.
fn print_text(opts: EchoOpts, io: &mut Io) -> CommandResult {
    match try_print_words(opts, io) {
        Ok(_) => CommandResult::code(status::SUCCESS),
        Err(error) => print_error(status::GENERAL_ERROR, &error.to_string(), io),
    }
}

/// Prints an error message to stderr and returns a status code.
fn print_error(status: i32, error: &str, io: &mut Io) -> CommandResult {
    let _ = writeln!(io.stderr, "{}: {}", NAME, error);
    CommandResult::code(status)
}

/// Tries to print words to stdout.
fn try_print_words(opts: EchoOpts, io: &mut Io) -> std::io::Result<()> {
    let mut words = opts.text.iter();

    // The first word should be written as-is.
    if let Some(word) = words.next() {
        write!(io.stdout, "{}", word)?;
    }

    // Remaining words are prefixed with a whitespace to ensure separation.
    for word in words {
        write!(io.stdout, " {}", word)?;
    }

    if !opts.no_newline {
        writeln!(io.stdout)?;
    }

    // Finally, flush the output stream to ensure that the output is displayed.
    io.stdout.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use pjsh_core::{Context, Scope};

    use crate::utils::{file_contents, mock_io};

    use super::*;

    #[test]
    fn it_prints_to_stdout() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["echo".into(), "message".into()]),
            None,
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        let (mut io, mut stdout, mut stderr) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let cmd = Echo {};
        if let CommandResult::Builtin(result) = cmd.run(&mut args) {
            assert_eq!(result.code, 0);
            assert!(result.actions.is_empty());
            assert_eq!(&file_contents(&mut stdout), "message\n");
            assert_eq!(&file_contents(&mut stderr), "");
        } else {
            unreachable!()
        }
    }

    #[test]
    fn it_separates_arguments_with_a_single_space() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["echo".into(), "first".into(), "second".into()]),
            None,
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        let (mut io, mut stdout, mut stderr) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let cmd = Echo {};
        if let CommandResult::Builtin(result) = cmd.run(&mut args) {
            assert_eq!(result.code, 0);
            assert!(result.actions.is_empty());
            assert_eq!(&file_contents(&mut stdout), "first second\n");
            assert_eq!(&file_contents(&mut stderr), "");
        } else {
            unreachable!()
        }
    }

    #[test]
    fn it_can_print_without_final_newline() {
        let mut ctx = Context::with_scopes(vec![Scope::new(
            String::new(),
            Some(vec!["echo".into(), "-n".into(), "message".into()]),
            None,
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        let (mut io, mut stdout, mut stderr) = mock_io();
        let mut args = Args::new(&mut ctx, &mut io);

        let cmd = Echo {};
        if let CommandResult::Builtin(result) = cmd.run(&mut args) {
            assert_eq!(result.code, 0);
            assert!(result.actions.is_empty());
            assert_eq!(&file_contents(&mut stdout), "message"); // No newline.
            assert_eq!(&file_contents(&mut stderr), "");
        } else {
            unreachable!()
        }
    }
}
