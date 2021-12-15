use sysinfo::{
    get_current_pid, Pid, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt,
};

use pjsh_core::Context;

/// Constructs a new initialized execution context containing some common environment variables such
/// as `$PS1` and `$PS2`.
pub fn initialized_context() -> Context {
    let context = Context::new();

    // Inject independent defaults.
    inject_static_defaults(&context);
    inject_shell_specific_env(&context);

    // Finally, inject external so that other defaults can be replaced.
    inject_external_envs(&context);

    context
}

/// Injects shell specific environment variables into a context.
fn inject_shell_specific_env(context: &Context) {
    if let Ok(exe) = std::env::current_exe().map(|path| path.to_string_lossy().to_string()) {
        context.scope.set_env(String::from("SHELL"), exe);
    }

    if let Ok(pwd) = std::env::current_dir().map(|dir| dir.to_string_lossy().to_string()) {
        context.scope.set_env(String::from("PWD"), pwd);
    }

    if let Some(home_dir) = dirs::home_dir().map(|path| path.to_string_lossy().to_string()) {
        context.scope.set_env(String::from("HOME"), home_dir);
    }

    // Parent process id.
    if let Ok(pid) = get_current_pid() {
        let system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
        );
        if let Some(process) = system.process(Pid::from(pid)) {
            if let Some(parent_id) = process.parent() {
                context
                    .scope
                    .set_env(String::from("PPID"), parent_id.to_string());
            }
        }
    }
}

/// Injects externally defined environment variables from the host into a context.
fn inject_external_envs(ctx: &Context) {
    for (key, value) in ctx.host.lock().env_vars() {
        ctx.scope.set_env(
            key.to_string_lossy().to_string(),
            value.to_string_lossy().to_string(),
        );
    }
}

/// Injects static default environment variables into a context.
fn inject_static_defaults(ctx: &Context) {
    ctx.scope.set_env(String::from("PS1"), String::from("$ "));
    ctx.scope.set_env(String::from("PS2"), String::from("> "));
    ctx.scope.set_env(String::from("PS4"), String::from("+ "));
}
