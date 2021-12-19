use std::path::PathBuf;

use crate::Context;

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
fn resolve_path() {
    // If $PWD is not set, "/" should be used instead.
    let empty_context = Context::default();
    assert_eq!(
        super::resolve_path(&empty_context, "relative"),
        PathBuf::from("/relative")
    );
    assert_eq!(
        super::resolve_path(&empty_context, "/absolute"),
        PathBuf::from("/absolute")
    );

    let linux_context = Context::default();
    linux_context
        .scope
        .set_env("PWD".into(), "/home/user".into());
    assert_eq!(
        super::resolve_path(&linux_context, "relative"),
        PathBuf::from("/home/user/relative")
    );
    assert_eq!(
        super::resolve_path(&linux_context, "/absolute"),
        PathBuf::from("/absolute")
    );

    #[cfg(target_os = "windows")]
    {
        let windows_context = Context::default();
        windows_context
            .scope
            .set_env("PWD".into(), r#"C:\\Dev"#.into());
        assert_eq!(
            super::resolve_path(&windows_context, "relative"),
            PathBuf::from(r#"C:\\Dev\relative"#)
        );
        assert_eq!(
            super::resolve_path(&windows_context, r#"D:\\absolute"#),
            PathBuf::from(r#"D:\\absolute"#)
        );
    }
}
