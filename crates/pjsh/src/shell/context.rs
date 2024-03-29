use std::{
    collections::{HashMap, HashSet},
    env::current_exe,
    path::PathBuf,
    sync::Arc,
};

use crate::{builtins::complete::Complete, source_file};
use parking_lot::Mutex;
use pjsh_complete::Completer;
use pjsh_core::{utils::path_to_string, Context, Filter, Scope, FD_STDERR, FD_STDIN, FD_STDOUT};

/// Constructs a new initialized execution context containing some common environment variables such
/// as `$PS1` and `$PS2`.
pub fn initialized_context(
    args: Vec<String>,
    script_file: Option<PathBuf>,
) -> (Context, Arc<Mutex<Completer>>) {
    let completer = Arc::new(Mutex::new(Completer::default()));

    let mut context = Context::with_scopes(vec![
        environment_scope(script_file.clone()),
        pjsh_scope(script_file),
        global_scope(args),
    ]);
    register_builtins(&mut context, Arc::clone(&completer));
    register_filters(&mut context);

    context.set_file_descriptor(FD_STDIN, pjsh_core::FileDescriptor::Stdin);
    context.set_file_descriptor(FD_STDOUT, pjsh_core::FileDescriptor::Stdout);
    context.set_file_descriptor(FD_STDERR, pjsh_core::FileDescriptor::Stderr);

    (context, completer)
}

/// Returns a scope containing all environment variables belonging to the
/// current process.
fn environment_scope(script_file: Option<PathBuf>) -> Scope {
    let mut vars: HashMap<String, Option<pjsh_core::Value>> = std::env::vars()
        .into_iter()
        .map(|(key, value)| (key, Some(pjsh_core::Value::Word(value))))
        .collect();

    // Inject the initial (current) script path if known and not already present.
    if !vars.contains_key("PJSH_INITIAL_SCRIPT_PATH") {
        if let Some(file) = script_file {
            let file = file.canonicalize().unwrap_or(file);
            vars.insert(
                "PJSH_INITIAL_SCRIPT_PATH".to_owned(),
                Some(pjsh_core::Value::Word(path_to_string(&file))),
            );
            if let Some(dir) = file.parent() {
                vars.insert(
                    "PJSH_INITIAL_SCRIPT_DIR".to_owned(),
                    Some(pjsh_core::Value::Word(path_to_string(dir))),
                );
            }
        }
    }

    // PWD is not known if the shell is started as a standalone process, but some
    // shell built-ins require it to work efficiently.
    if !vars.contains_key("PWD") {
        if let Ok(pwd) = std::env::current_dir() {
            vars.insert(
                "PWD".to_owned(),
                Some(pjsh_core::Value::Word(path_to_string(pwd))),
            );
        }
    }

    Scope::new(
        "environment".to_owned(),
        None,
        vars,
        HashMap::default(),
        HashSet::default(),
    )
}

/// Returns a scope containing shell-specific default variables.
fn pjsh_scope(script_file: Option<PathBuf>) -> Scope {
    let mut vars = HashMap::from([
        (
            "PS1".to_owned(),
            Some(pjsh_core::Value::Word("\\$ ".to_owned())),
        ),
        (
            "PS2".to_owned(),
            Some(pjsh_core::Value::Word("> ".to_owned())),
        ),
        (
            "PS4".to_owned(),
            Some(pjsh_core::Value::Word("+ ".to_owned())),
        ),
    ]);

    // Inject the current script path if known.
    if let Some(file) = script_file {
        let file = file.canonicalize().unwrap_or(file);
        vars.insert(
            "PJSH_CURRENT_SCRIPT_PATH".to_owned(),
            Some(pjsh_core::Value::Word(path_to_string(&file))),
        );
        if let Some(dir) = file.parent() {
            vars.insert(
                "PJSH_CURRENT_SCRIPT_DIR".to_owned(),
                Some(pjsh_core::Value::Word(path_to_string(dir))),
            );
        }
    }

    Scope::new(
        "pjsh".to_owned(),
        None,
        vars,
        HashMap::default(),
        HashSet::default(),
    )
}

/// Returns an empty scope for use as the shell's global scope.
fn global_scope(args: Vec<String>) -> Scope {
    let name = current_exe().map_or_else(|_| String::from("pjsh"), path_to_string);

    Scope::new(
        name,
        Some(args),
        HashMap::default(),
        HashMap::default(),
        HashSet::default(),
    )
}

/// Registers built-in commands in a context.
fn register_builtins(context: &mut Context, completer: Arc<Mutex<Completer>>) {
    context.register_builtin(Box::new(pjsh_builtins::Alias));
    context.register_builtin(Box::new(pjsh_builtins::Cd));
    context.register_builtin(Box::new(Complete::new(completer)));
    context.register_builtin(Box::new(pjsh_builtins::Echo));
    context.register_builtin(Box::new(pjsh_builtins::Exit));
    context.register_builtin(Box::new(pjsh_builtins::Export));
    context.register_builtin(Box::new(pjsh_builtins::False));
    context.register_builtin(Box::new(pjsh_builtins::Interpolate));
    context.register_builtin(Box::new(pjsh_builtins::Pwd));
    context.register_builtin(Box::new(pjsh_builtins::Sleep));
    context.register_builtin(Box::new(pjsh_builtins::Source::new(source_file)));
    context.register_builtin(Box::new(pjsh_builtins::SourceShorthand::new(source_file)));
    context.register_builtin(Box::new(pjsh_builtins::True));
    context.register_builtin(Box::new(pjsh_builtins::Type));
    context.register_builtin(Box::new(pjsh_builtins::Unalias));
    context.register_builtin(Box::new(pjsh_builtins::Unset));
    context.register_builtin(Box::new(pjsh_builtins::Which));
}

/// Registers built-in filters in a context.
fn register_filters(context: &mut Context) {
    let register = |context: &mut Context, filter: Box<dyn Filter>| {
        let previous = context.filters.insert(filter.name().to_string(), filter);
        assert!(previous.is_none(), "filters should not be registered twice");
    };

    register(context, Box::new(pjsh_filters::FirstFilter));
    register(context, Box::new(pjsh_filters::JoinFilter));
    register(context, Box::new(pjsh_filters::LastFilter));
    register(context, Box::new(pjsh_filters::LenFilter));
    register(context, Box::new(pjsh_filters::LinesFilter));
    register(context, Box::new(pjsh_filters::LowercaseFilter));
    register(context, Box::new(pjsh_filters::NthFilter));
    register(context, Box::new(pjsh_filters::ReplaceFilter));
    register(context, Box::new(pjsh_filters::ReverseFilter));
    register(context, Box::new(pjsh_filters::SortFilter));
    register(context, Box::new(pjsh_filters::SplitFilter));
    register(context, Box::new(pjsh_filters::UcfirstFilter));
    register(context, Box::new(pjsh_filters::UniqueFilter));
    register(context, Box::new(pjsh_filters::UppercaseFilter));
    register(context, Box::new(pjsh_filters::WordsFilter));
}

#[cfg(test)]
mod tests {
    use pjsh_core::Value;

    use super::*;

    #[test]
    fn it_registers_builtins() {
        let expected_builtins = vec![
            ".",
            "alias",
            "cd",
            "complete",
            "echo",
            "exit",
            "export",
            "false",
            "interpolate",
            "pwd",
            "sleep",
            "source",
            "true",
            "type",
            "unalias",
            "unset",
            "which",
        ];

        let (context, _) = initialized_context(Vec::new(), None);
        let mut builtins: Vec<&str> = context
            .builtins
            .keys()
            .map(|builtin| builtin.as_str())
            .collect();
        builtins.sort();

        assert_eq!(builtins, expected_builtins);
    }

    #[test]
    fn it_registers_script_path_variables() {
        let script_file = PathBuf::from("/tmp/test_script.pjsh");
        let (context, _) = initialized_context(Vec::new(), Some(script_file));

        // Initial script paths should be set.
        assert_eq!(
            context.get_var("PJSH_INITIAL_SCRIPT_PATH"),
            Some(&Value::Word("/tmp/test_script.pjsh".into()))
        );
        assert_eq!(
            context.get_var("PJSH_INITIAL_SCRIPT_DIR"),
            Some(&Value::Word("/tmp".into()))
        );

        // Current script paths should be set.
        assert_eq!(
            context.get_var("PJSH_CURRENT_SCRIPT_PATH"),
            Some(&Value::Word("/tmp/test_script.pjsh".into()))
        );
        assert_eq!(
            context.get_var("PJSH_CURRENT_SCRIPT_DIR"),
            Some(&Value::Word("/tmp".into()))
        );
    }
}
