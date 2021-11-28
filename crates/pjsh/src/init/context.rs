use pjsh_core::Context;

pub fn init_context() -> Context {
    let context = Context::new();
    for (key, value) in context.host.lock().env_vars() {
        context.scope.set_env(
            key.to_string_lossy().to_string(),
            value.to_string_lossy().to_string(),
        );
    }

    if let Ok(exe) = std::env::current_exe().map(|path| path.to_string_lossy().to_string()) {
        context.scope.set_env(String::from("SHELL"), exe);
    }

    if let Ok(pwd) = std::env::current_dir().map(|dir| dir.to_string_lossy().to_string()) {
        context.scope.set_env(String::from("PWD"), pwd);
    }

    if let Some(home_dir) = dirs::home_dir().map(|path| path.to_string_lossy().to_string()) {
        context.scope.set_env(String::from("HOME"), home_dir);
    }

    context
}
