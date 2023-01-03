use std::sync::Arc;

use clap::Parser;
use parking_lot::Mutex;
use pjsh_builtins::exit_with_parse_error;
use pjsh_complete::{Completer, Completion};
use pjsh_core::{
    command::Args,
    command::{Command, CommandResult},
};

/// Command name.
const NAME: &str = "complete";

/// Status code indicating successful command execution.
const SUCCESS: i32 = 0;

/// Status code indicating a general error during command execution.
const GENERAL_ERROR: i32 = 1;

/// Define shell completions.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct CompleteOpts {
    /// Name for which completions exist.
    name: String,

    /// A pre-defined action.
    #[clap(short = 'A')]
    action: Option<String>,

    /// A function to call in order to retrieve completions.
    #[clap(short = 'F')]
    function: Option<String>,

    /// A fixed list of words.
    #[clap(short = 'W')]
    wordlist: Option<String>,
}

/// Implementation for the "complete" built-in command.
#[derive(Clone)]
pub struct Complete {
    /// Shell completions.
    completer: Arc<Mutex<Completer>>,
}

impl Complete {
    /// Constructs a new completion command.
    pub fn new(completer: Arc<Mutex<Completer>>) -> Self {
        Self { completer }
    }
}

impl Command for Complete {
    fn name(&self) -> &str {
        "complete"
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match CompleteOpts::try_parse_from(args.context.args()) {
            Ok(opts) => {
                let mut completer = self.completer.lock();

                if let Some(action) = opts.action {
                    let completion = match action.as_str() {
                        "directory" => Completion::Directory,
                        "file" => Completion::File,
                        _ => {
                            let _ = writeln!(args.io.stderr, "Unknown action: {action}");
                            return CommandResult::code(GENERAL_ERROR);
                        }
                    };
                    completer.register_completion(opts.name, completion);
                    return CommandResult::code(SUCCESS);
                }

                if let Some(function) = opts.function {
                    completer.register_completion(opts.name, Completion::Function(function));
                    return CommandResult::code(SUCCESS);
                }

                if let Some(wordlist) = opts.wordlist {
                    completer.register_completion(opts.name, Completion::Constant(words(wordlist)));
                }

                CommandResult::code(SUCCESS)
            }
            Err(error) => exit_with_parse_error(args.io, error),
        }
    }
}

/// Returns a `Vec<String>` of all whitespace-separated words in a string.
fn words(wordlist: String) -> Vec<String> {
    wordlist
        .split_whitespace()
        .map(|word| word.to_string())
        .collect()
}
