use pjsh_exec::Executor;

/// Creates a new executor with registered built-in commands.
pub(crate) fn create_executor() -> Executor {
    Executor::new(vec![
        Box::new(pjsh_builtins::Alias),
        Box::new(pjsh_builtins::Cd),
        Box::new(pjsh_builtins::Echo),
        Box::new(pjsh_builtins::Exit),
        Box::new(pjsh_builtins::False),
        Box::new(pjsh_builtins::Interpolate),
        Box::new(pjsh_builtins::Pwd),
        Box::new(pjsh_builtins::Sleep),
        Box::new(pjsh_builtins::Source),
        Box::new(pjsh_builtins::True),
        Box::new(pjsh_builtins::Type),
        Box::new(pjsh_builtins::Unalias),
        Box::new(pjsh_builtins::Unset),
        Box::new(pjsh_builtins::Which),
    ])
}
