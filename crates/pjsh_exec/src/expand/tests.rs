use std::collections::VecDeque;

use mockall::mock;

use pjsh_ast::Word;
use pjsh_core::{Context, Host};

use crate::expand::{alias::expand_aliases, glob::expand_globs};
use crate::tests::utils::test_executor;

use super::expand_single;

mock! {
    TestHost {}
    impl Host for TestHost {
        fn println(&mut self, text: &str);
        fn eprintln(&mut self, text: &str);
        fn add_child_process(&mut self, child: std::process::Child);
        fn add_thread(&mut self, thread: std::thread::JoinHandle<i32>);
        fn kill_all_processes(&mut self);
        fn process_id(&self) -> u32;
        fn join_all_threads(&mut self);
        fn take_exited_child_processes(&mut self) -> std::collections::HashSet<u32>;
        fn env_vars(&self) -> std::collections::HashMap<std::ffi::OsString, std::ffi::OsString>;
        fn get_env(&self, key: &std::ffi::OsStr) -> Option<std::ffi::OsString>;
        fn set_env(&mut self, key: std::ffi::OsString, value: std::ffi::OsString);
        fn unset_env(&mut self, key: &std::ffi::OsStr);
    }
}

#[test]
fn expand_single_alias() {
    let context = Context::default();
    context.scope.set_alias("cmd1".into(), "cmd2".into());
    let mut words = VecDeque::from(vec![("cmd1".into(), true), ("args".into(), true)]);
    expand_aliases(&mut words, &context);
    assert_eq!(
        words,
        VecDeque::from(vec![("cmd2".into(), true), ("args".into(), true)])
    )
}

#[test]
fn expand_recursive_alias() {
    let context = Context::default();
    context.scope.set_alias("cmd1".into(), "cmd2".into());
    context.scope.set_alias("cmd2".into(), "cmd3".into());
    // Expand cmd1 -> cmd2 -> cmd3.
    let mut words = VecDeque::from(vec![("cmd1".into(), true), ("args".into(), true)]);
    expand_aliases(&mut words, &context);
    assert_eq!(
        words,
        VecDeque::from(vec![("cmd3".into(), true), ("args".into(), true)])
    )
}

#[test]
fn expand_only_first_word_alias() {
    let context = Context::default();
    context.scope.set_alias("aliased".into(), "expanded".into());
    let mut words = VecDeque::from(vec![("aliased".into(), true), ("aliased".into(), true)]);
    expand_aliases(&mut words, &context);
    assert_eq!(
        words,
        VecDeque::from(vec![("expanded".into(), true), ("aliased".into(), true)])
    )
}

#[test]
fn stop_alias_expansion_on_whitespace_ending_alias() {
    let context = Context::default();
    context.scope.set_alias("cmd1".into(), "cmd2 ".into()); // Ends with whitespace.
    context.scope.set_alias("cmd2".into(), "cmd3".into());
    let mut words = VecDeque::from(vec![("cmd1".into(), true), ("args".into(), true)]);
    expand_aliases(&mut words, &context);
    assert_eq!(
        words,
        VecDeque::from(vec![("cmd2".into(), true), ("args".into(), true)])
    )
}

#[test]
fn stop_alias_expansion_on_duplicate() {
    let context = Context::default();
    context.scope.set_alias("cmd1".into(), "cmd2".into());
    context.scope.set_alias("cmd2".into(), "cmd1".into());
    // Expand cmd1 -> cmd2 -> cmd1 (duplicate).
    let mut words = VecDeque::from(vec![("cmd1".into(), true), ("args".into(), true)]);
    expand_aliases(&mut words, &context);
    assert_eq!(
        words,
        VecDeque::from(vec![("cmd1".into(), true), ("args".into(), true)])
    )
}

#[test]
fn expand_tilde() {
    let context = Context::default();
    context.scope.set_env("HOME".into(), "HOME".into());
    let mut words = VecDeque::from(vec![
        ("~".into(), true),
        ("~/.pjsh".into(), true),
        ("file~".into(), true),
    ]);
    expand_globs(&mut words, &context);

    assert_eq!(
        words,
        VecDeque::from(vec![
            ("HOME".into(), true),
            ("HOME/.pjsh".into(), true),
            ("file~".into(), true)
        ])
    )
}

#[test]
fn expand_dollar() {
    let mut context = Context::default();
    let executor = test_executor();
    let mut mock_host = MockTestHost::new();
    mock_host.expect_process_id().returning(|| 4444);
    context.host = std::sync::Arc::new(parking_lot::Mutex::new(mock_host));

    let word = expand_single(Word::Variable("$".into()), &context, &executor);
    assert_eq!(word, Some("4444".into()));
}
