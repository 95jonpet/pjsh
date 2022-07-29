use std::collections::{HashMap, HashSet};

use pjsh_core::{utils::path_to_string, Context, Host, Scope, StdHost};

/// Constructs a new initialized execution context containing some common environment variables such
/// as `$PS1` and `$PS2`.
pub fn initialized_context(interactive: bool) -> Context {
    let host = StdHost::default();
    Context::with_scopes(vec![
        environment_scope(host),
        pjsh_scope(interactive),
        global_scope(interactive),
    ])
}

/// Returns a scope containing all environment variables belonging to the
/// current process.
fn environment_scope<H: Host>(host: H) -> Scope {
    let vars: HashMap<String, String> = host
        .env_vars()
        .iter()
        .map(|(key, value)| {
            (
                key.to_string_lossy().to_string(),
                value.to_string_lossy().to_string(),
            )
        })
        .collect();

    Scope::new(
        "evironment".to_owned(),
        None,
        Some(vars),
        None,
        HashSet::default(),
        false,
    )
}

/// Returns a scope containing shell-specific default variables.
fn pjsh_scope(interactive: bool) -> Scope {
    let vars = HashMap::from([
        ("PS1".to_owned(), "\\$ ".to_owned()),
        ("PS2".to_owned(), "> ".to_owned()),
        ("PS4".to_owned(), "+ ".to_owned()),
    ]);

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
fn global_scope(interactive: bool) -> Scope {
    let name = std::env::current_exe()
        .map(|path| path_to_string(&path))
        .unwrap_or_else(|_| String::from("pjsh"));

    Scope::new(
        name,
        Some(vec![]), // TODO: Initialize arguments from argument list.
        Some(HashMap::default()),
        Some(HashMap::default()),
        HashSet::default(),
        interactive,
    )
}
