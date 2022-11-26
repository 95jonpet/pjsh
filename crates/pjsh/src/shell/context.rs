use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use pjsh_core::{
    utils::path_to_string, Context, Host, Scope, StdHost, FD_STDERR, FD_STDIN, FD_STDOUT,
};

/// Constructs a new initialized execution context containing some common environment variables such
/// as `$PS1` and `$PS2`.
pub fn initialized_context(
    args: Vec<String>,
    script_file: Option<PathBuf>,
    interactive: bool,
) -> Context {
    let host = StdHost::default();
    let mut context = Context::with_scopes(vec![
        environment_scope(host, script_file.clone()),
        pjsh_scope(script_file, interactive),
        global_scope(args, interactive),
    ]);
    register_builtins(&mut context);

    context.set_file_descriptor(FD_STDIN, pjsh_core::FileDescriptor::Stdin);
    context.set_file_descriptor(FD_STDOUT, pjsh_core::FileDescriptor::Stdout);
    context.set_file_descriptor(FD_STDERR, pjsh_core::FileDescriptor::Stderr);

    context
}

/// Returns a scope containing all environment variables belonging to the
/// current process.
fn environment_scope<H: Host>(host: H, script_file: Option<PathBuf>) -> Scope {
    let mut vars: HashMap<String, String> = host
        .env_vars()
        .iter()
        .map(|(key, value)| {
            (
                key.to_string_lossy().to_string(),
                value.to_string_lossy().to_string(),
            )
        })
        .collect();

    // Inject the initial (current) script path if known and not already present.
    if !vars.contains_key("PJSH_INITIAL_SCRIPT_PATH") {
        if let Some(file) = script_file {
            let file = file.canonicalize().unwrap_or(file);
            vars.insert("PJSH_INITIAL_SCRIPT_PATH".to_owned(), path_to_string(&file));
            if let Some(dir) = file.parent() {
                vars.insert("PJSH_INITIAL_SCRIPT_DIR".to_owned(), path_to_string(dir));
            }
        }
    }

    // PWD is not known if the shell is started as a standalone process, but some
    // shell built-ins require it to work efficiently.
    if !vars.contains_key("PWD") {
        if let Ok(pwd) = std::env::current_dir() {
            vars.insert("PWD".to_owned(), path_to_string(pwd));
        }
    }

    Scope::new(
        "environment".to_owned(),
        None,
        Some(vars),
        None,
        HashSet::default(),
        false,
    )
}

/// Returns a scope containing shell-specific default variables.
fn pjsh_scope(script_file: Option<PathBuf>, interactive: bool) -> Scope {
    let mut vars = HashMap::from([
        ("PS1".to_owned(), "\\$ ".to_owned()),
        ("PS2".to_owned(), "> ".to_owned()),
        ("PS4".to_owned(), "+ ".to_owned()),
    ]);

    // Inject the current script path if known.
    if let Some(file) = script_file {
        let file = file.canonicalize().unwrap_or(file);
        vars.insert("PJSH_CURRENT_SCRIPT_PATH".to_owned(), path_to_string(&file));
        if let Some(dir) = file.parent() {
            vars.insert("PJSH_CURRENT_SCRIPT_DIR".to_owned(), path_to_string(dir));
        }
    }

    Scope::new(
        "pjsh".to_owned(),
        None,
        Some(vars),
        None,
        HashSet::default(),
        interactive,
    )
}

/// Returns an empty scope for use as the shell's global scope.
fn global_scope(args: Vec<String>, interactive: bool) -> Scope {
    let name = std::env::current_exe()
        .map(|path| path_to_string(&path))
        .unwrap_or_else(|_| String::from("pjsh"));

    Scope::new(
        name,
        Some(args),
        Some(HashMap::default()),
        Some(HashMap::default()),
        HashSet::default(),
        interactive,
    )
}

/// Registers built-in commands in a context.
fn register_builtins(context: &mut Context) {
    context.register_builtin(Box::new(pjsh_builtins::Alias));
    context.register_builtin(Box::new(pjsh_builtins::Cd));
    context.register_builtin(Box::new(pjsh_builtins::Echo));
    context.register_builtin(Box::new(pjsh_builtins::Exit));
    context.register_builtin(Box::new(pjsh_builtins::Export));
    context.register_builtin(Box::new(pjsh_builtins::False));
    context.register_builtin(Box::new(pjsh_builtins::Interpolate));
    context.register_builtin(Box::new(pjsh_builtins::Pwd));
    context.register_builtin(Box::new(pjsh_builtins::Sleep));
    context.register_builtin(Box::new(pjsh_builtins::Source));
    context.register_builtin(Box::new(pjsh_builtins::SourceShorthand));
    context.register_builtin(Box::new(pjsh_builtins::True));
    context.register_builtin(Box::new(pjsh_builtins::Type));
    context.register_builtin(Box::new(pjsh_builtins::Unalias));
    context.register_builtin(Box::new(pjsh_builtins::Unset));
    context.register_builtin(Box::new(pjsh_builtins::Which));
}
