use std::path::PathBuf;

use super::*;
use crate::{env::context::Value, Context};

#[test]
fn path_to_string() {
    assert_eq!(
        &super::path_to_string(&PathBuf::from(r#"C:\\Dev"#)),
        r#"C:\\Dev"#
    );
    assert_eq!(
        &super::path_to_string(&PathBuf::from("/usr/bin")),
        "/usr/bin"
    );
}

#[test]
fn test_resolve_path_with_empty_context() {
    // If $PWD is not set, "/" should be used instead.
    let ctx = Context::default();
    assert_eq!(resolve_path(&ctx, "relative"), PathBuf::from("/relative"));
    assert_eq!(resolve_path(&ctx, "/absolute"), PathBuf::from("/absolute"));
}

#[test]
fn test_resolve_path_with_linux_pwd_context() {
    let mut ctx = Context::default();
    ctx.set_var("PWD".into(), Value::Word("/base".into()));
    assert_eq!(resolve_path(&ctx, "child"), PathBuf::from("/base/child"));
    assert_eq!(resolve_path(&ctx, "/absolute"), PathBuf::from("/absolute"));
}
