use std::collections::{HashMap, HashSet};

use sysinfo::{get_current_pid, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};

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
        vec![],
        vars,
        HashMap::default(),
        HashSet::default(),
        false,
    )
}

/// Returns a scope containing shell-specific default variables.
fn pjsh_scope(interactive: bool) -> Scope {
    let mut vars = HashMap::from([
        ("PS1".to_owned(), "\\$ ".to_owned()),
        ("PS2".to_owned(), "> ".to_owned()),
        ("PS4".to_owned(), "+ ".to_owned()),
    ]);

    if let Ok(exe) = std::env::current_exe().map(|path| path_to_string(&path)) {
        vars.insert("SHELL".to_owned(), exe);
    }

    if let Ok(pwd) = std::env::current_dir().map(|path| path_to_string(&path)) {
        vars.insert("PWD".to_owned(), pwd);
    }

    if let Some(home_dir) = dirs::home_dir().map(|path| path_to_string(&path)) {
        vars.insert("HOME".to_owned(), home_dir);
    }

    // Parent process id.
    if let Ok(pid) = get_current_pid() {
        let system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
        );
        if let Some(process) = system.process(pid) {
            if let Some(parent_id) = process.parent() {
                vars.insert("PPID".to_owned(), parent_id.to_string());
            }
        }
    }

    Scope::new(
        "pjsh".to_owned(),
        vec![],
        vars,
        HashMap::default(),
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
        vec![],
        HashMap::default(),
        HashMap::default(),
        HashSet::default(),
        interactive,
    )
}
