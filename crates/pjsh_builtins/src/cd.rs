use std::{ffi::OsString, path::PathBuf};

use clap::Parser;
use pjsh_core::{
    command::Io,
    command::{Args, Command, CommandResult},
    utils::{path_to_string, resolve_path},
    Context,
};

use crate::{status, utils};

/// Command name.
const NAME: &str = "cd";

/// Change the shell's working directory.
///
/// If no directory is supplied, user's home directory is used.
///
/// This is a built-in shell command.
#[derive(Parser)]
#[clap(name = NAME, version)]
struct CdOpts {
    /// Directory to change to.
    ///
    /// If supplied with the directory "-", the working directory is changed to
    /// the shell's previous working directory.
    directory: Option<OsString>,
}

/// Implementation for the "echo" built-in command.
#[derive(Clone)]
pub struct Cd;
impl Command for Cd {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, mut args: Args) -> CommandResult {
        let mut ctx = args.context.lock();
        match CdOpts::try_parse_from(ctx.args()) {
            Ok(opts) => change_directory(opts, &mut ctx, &mut args.io),
            Err(err) => utils::exit_with_parse_error(&mut args.io, err),
        }
    }
}

/// Changes the current working directory of a context.
///
/// Prints the new working directory to stdout if the directory is "-".
///
/// Returns an exit code.
fn change_directory(opts: CdOpts, ctx: &mut Context, io: &mut Io) -> CommandResult {
    let directory = match &opts.directory {
        Some(dir) if dir == "-" => ctx.get_var("OLDPWD").map(PathBuf::from),
        Some(dir) => Some(resolve_path(ctx, dir)),
        None => ctx.get_var("HOME").map(PathBuf::from),
    };

    match directory {
        Some(path) => {
            // Ensure that the requested directory path is a valid directory.
            if !path.is_dir() {
                return exit_with_error(status::GENERAL_ERROR, io, "Path is not a directory.");
            }

            // Keep track of the old working directory within the context.
            if let Some(pwd) = ctx.get_var("PWD").map(|pwd| pwd.to_owned()) {
                ctx.set_var("OLDPWD".to_owned(), pwd);
            }

            // Set the current working directory within the current context.
            let new_path = path_to_string(&path);
            ctx.set_var("PWD".to_string(), new_path.clone());

            // Using "-" as a directory should be equivalent to "cd - && pwd".
            if opts.directory.filter(|p| p == "-").is_some() {
                if let Err(err) = writeln!(io.stdout, "{}", &new_path) {
                    return exit_with_error(status::GENERAL_ERROR, io, &err.to_string());
                }
            }

            CommandResult::code(status::SUCCESS)
        }
        None => exit_with_error(status::GENERAL_ERROR, io, "No directory to change to."),
    }
}

/// Prints an error message to standard error.
///
/// Returns an exit code.
fn exit_with_error(status: i32, io: &mut Io, error: &str) -> CommandResult {
    let _ = writeln!(io.stderr, "{}: {}", NAME, error);
    CommandResult::code(status)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        path::Path,
        sync::Arc,
    };

    use parking_lot::Mutex;
    use pjsh_core::{utils::path_to_string, Scope};
    use tempfile::TempDir;

    use crate::utils::{file_contents, mock_io};

    use super::*;

    /// Constructs a context for the `cd path` command, where `path` is read from an
    /// argument.
    fn cd_context<P: AsRef<Path>>(path: P) -> Arc<Mutex<Context>> {
        let context = Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["cd".into(), path_to_string(&path)],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )]);
        Arc::new(Mutex::new(context))
    }

    #[test]
    fn it_can_change_working_directory() {
        let dir = TempDir::new().unwrap();
        let ctx = cd_context(&dir);
        ctx.lock().set_var("PWD".into(), "old-pwd".into());
        let (io, _stdout, _stderr) = mock_io();
        let cd = Cd {};

        let args = Args::new(Arc::clone(&ctx), io);
        let result = cd.run(args);

        assert_eq!(result.code, status::SUCCESS);
        assert_eq!(
            ctx.lock().get_var("PWD"),
            Some(path_to_string(dir.path()).as_str())
        );
        assert_eq!(ctx.lock().get_var("OLDPWD"), Some("old-pwd".into()));
    }

    #[test]
    fn it_can_change_working_directory_to_home() {
        let home = TempDir::new().unwrap();
        let ctx = Arc::new(Mutex::new(Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["cd".into()],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )])));
        ctx.lock().set_var("HOME".into(), path_to_string(&home));
        let (io, _stdout, _stderr) = mock_io();
        let cd = Cd {};

        let args = Args::new(Arc::clone(&ctx), io);
        let result = cd.run(args);

        assert_eq!(result.code, status::SUCCESS);
        assert_eq!(
            ctx.lock().get_var("PWD"),
            Some(path_to_string(home).as_str())
        );
    }

    #[test]
    fn it_can_change_working_directory_to_oldpwd() {
        let oldpwd = TempDir::new().unwrap();
        let ctx = Arc::new(Mutex::new(Context::with_scopes(vec![Scope::new(
            String::new(),
            vec!["cd".into(), "-".into()],
            HashMap::default(),
            HashMap::default(),
            HashSet::default(),
            false,
        )])));
        ctx.lock().set_var("OLDPWD".into(), path_to_string(&oldpwd));
        let (io, mut stdout, _stderr) = mock_io();
        let cd = Cd {};

        let args = Args::new(Arc::clone(&ctx), io);
        let result = cd.run(args);

        assert_eq!(result.code, status::SUCCESS);
        assert_eq!(
            ctx.lock().get_var("PWD"),
            Some(path_to_string(&oldpwd).as_str())
        );
        assert_eq!(file_contents(&mut stdout), path_to_string(&oldpwd) + "\n");
    }

    #[test]
    fn it_cannot_change_working_directory_to_missing_directories() {
        let dir = PathBuf::from("/path/to/missing/dir");
        let ctx = cd_context(&dir);
        let (io, _stdout, _stderr) = mock_io();
        let cd = Cd {};

        let args = Args::new(Arc::clone(&ctx), io);
        let result = cd.run(args);

        assert_eq!(result.code, status::GENERAL_ERROR);
        assert_eq!(ctx.lock().get_var("PWD"), None);
    }
}
