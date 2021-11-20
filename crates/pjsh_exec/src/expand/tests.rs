use std::collections::VecDeque;

use pjsh_core::Context;

use crate::expand::{alias::expand_aliases, glob::expand_globs};

#[test]
fn expand_single_alias() {
    let context = Context::default();
    context.scope.set_alias("cmd1".into(), "cmd2".into());
    let mut words = VecDeque::from(vec!["cmd1".into(), "args".into()]);
    expand_aliases(&mut words, &context);
    assert_eq!(words, VecDeque::from(vec!["cmd2".into(), "args".into()]))
}

#[test]
fn expand_recursive_alias() {
    let context = Context::default();
    context.scope.set_alias("cmd1".into(), "cmd2".into());
    context.scope.set_alias("cmd2".into(), "cmd3".into());
    // Expand cmd1 -> cmd2 -> cmd3.
    let mut words = VecDeque::from(vec!["cmd1".into(), "args".into()]);
    expand_aliases(&mut words, &context);
    assert_eq!(words, VecDeque::from(vec!["cmd3".into(), "args".into()]))
}

#[test]
fn expand_only_first_word_alias() {
    let context = Context::default();
    context.scope.set_alias("aliased".into(), "expanded".into());
    let mut words = VecDeque::from(vec!["aliased".into(), "aliased".into()]);
    expand_aliases(&mut words, &context);
    assert_eq!(
        words,
        VecDeque::from(vec!["expanded".into(), "aliased".into()])
    )
}

#[test]
fn stop_alias_expansion_on_whitespace_ending_alias() {
    let context = Context::default();
    context.scope.set_alias("cmd1".into(), "cmd2 ".into()); // Ends with whitespace.
    context.scope.set_alias("cmd2".into(), "cmd3".into());
    let mut words = VecDeque::from(vec!["cmd1".into(), "args".into()]);
    expand_aliases(&mut words, &context);
    assert_eq!(words, VecDeque::from(vec!["cmd2".into(), "args".into()]))
}

#[test]
fn stop_alias_expansion_on_duplicate() {
    let context = Context::default();
    context.scope.set_alias("cmd1".into(), "cmd2".into());
    context.scope.set_alias("cmd2".into(), "cmd1".into());
    // Expand cmd1 -> cmd2 -> cmd1 (duplicate).
    let mut words = VecDeque::from(vec!["cmd1".into(), "args".into()]);
    expand_aliases(&mut words, &context);
    assert_eq!(words, VecDeque::from(vec!["cmd1".into(), "args".into()]))
}

#[test]
fn expand_tilde() {
    let context = Context::default();
    context.scope.set_env("HOME".into(), "HOME".into());
    let mut words = VecDeque::from(vec!["~".into(), "~/.pjsh".into(), "file~".into()]);
    expand_globs(&mut words, &context);

    assert_eq!(
        words,
        VecDeque::from(vec!["HOME".into(), "HOME/.pjsh".into(), "file~".into()])
    )
}
