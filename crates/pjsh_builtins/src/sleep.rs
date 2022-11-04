use clap::Parser;
use pjsh_core::command::{Args, Command, CommandResult};

use crate::{status, utils};

/// Command name.
const NAME: &str = "sleep";

/// Time unit for a sleep duration.
#[derive(Clone, clap::ArgEnum)]
enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
}

/// Wait for some time to pass.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct SleepOpts {
    /// Duration to sleep.
    duration: u64,

    /// Time unit for sleep the duration.
    #[clap(arg_enum, default_value = "seconds")]
    unit: TimeUnit,
}

/// Implementation for the "sleep" built-in command.
#[derive(Clone)]
pub struct Sleep;
impl Command for Sleep {
    fn name(&self) -> &str {
        NAME
    }

    fn run<'a>(&self, args: &'a mut Args) -> CommandResult {
        match SleepOpts::try_parse_from(args.context.args()) {
            Ok(opts) => sleep(opts),
            Err(error) => utils::exit_with_parse_error(args.io, error),
        }
    }
}

/// Sleep on the current thread for a while.
///
/// This method wraps [`std::thread::sleep`].
fn sleep(args: SleepOpts) -> CommandResult {
    // Exit early to avoid platform-specific system calls in std::thread::sleep.
    if args.duration == 0 {
        return CommandResult::code(status::SUCCESS);
    }

    let duration = parse_duration(&args);
    std::thread::sleep(duration);
    CommandResult::code(status::SUCCESS)
}

/// Parses a [`std::time::Duration`] from [`SleepOpts`].
fn parse_duration(args: &SleepOpts) -> std::time::Duration {
    match args.unit {
        TimeUnit::Seconds => std::time::Duration::from_secs(args.duration),
        TimeUnit::Minutes => std::time::Duration::from_secs(args.duration * 60),
        TimeUnit::Hours => std::time::Duration::from_secs(args.duration * 3600),
    }
}
