use std::path::PathBuf;

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
