use std::sync::Arc;

use clap::Parser;
use parking_lot::Mutex;
use pjsh_core::{
    command::Args,
    command::{Command, CommandResult},
    Completion, Completions,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "complete";

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
    completions: Arc<Mutex<Completions>>,
}

impl Complete {
    /// Constructs a new completion command.
    pub fn new(completions: Arc<Mutex<Completions>>) -> Self {
        Self { completions }
    }
}

impl Command for Complete {
    fn name(&self) -> &str {
        "complete"
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match CompleteOpts::try_parse_from(args.context.args()) {
            Ok(opts) => {
                if let Some(action) = opts.action {
                    let completion = match action.as_str() {
                        "directory" => Completion::Directory,
                        "file" => Completion::File,
                        _ => {
                            let _ = writeln!(args.io.stderr, "Unknown action: {action}");
                            return CommandResult::code(status::GENERAL_ERROR);
                        }
                    };
                    self.completions.lock().insert(opts.name, completion);
                    return CommandResult::code(status::SUCCESS);
                }

                if let Some(function) = opts.function {
                    self.completions
                        .lock()
                        .insert(opts.name, Completion::Function(function));
                    return CommandResult::code(status::SUCCESS);
                }

                if let Some(wordlist) = opts.wordlist {
                    self.completions
                        .lock()
                        .insert(opts.name, Completion::Constant(words(wordlist)));
                }

                CommandResult::code(status::SUCCESS)
            }
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

fn words(wordlist: String) -> Vec<String> {
    wordlist
        .split_whitespace()
        .map(|word| word.to_string())
        .collect()
}
