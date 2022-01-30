use std::path::PathBuf;

use pjsh_core::{command::Args, Condition};

/// A condition that is met when a given path exists.
#[derive(Clone)]
pub struct Exists(pub PathBuf);
impl Condition for Exists {
    fn evaluate(&self, _: Args) -> bool {
        self.0.exists()
    }
}

/// A condition that is met when a given path points to a file.
#[derive(Clone)]
pub struct IsFile(pub PathBuf);
impl Condition for IsFile {
    fn evaluate(&self, _: Args) -> bool {
        self.0.is_file()
    }
}

/// A condition that is met when a given path points to a directory.
#[derive(Clone)]
pub struct IsDirectory(pub PathBuf);
impl Condition for IsDirectory {
    fn evaluate(&self, _: Args) -> bool {
        self.0.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn args() -> Args {
        Args {
            context: pjsh_core::Context::default(),
            io: pjsh_core::command::Io {
                stdin: Box::new(std::io::empty()),
                stdout: Box::new(std::io::sink()),
                stderr: Box::new(std::io::sink()),
            },
        }
    }

    #[test]
    fn exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file");
        let _ = std::fs::File::create(file_path.clone());
        assert!(Exists(file_path).evaluate(args()));
        assert!(Exists(dir.path().to_path_buf()).evaluate(args()));
        assert!(!Exists(PathBuf::from("/non/existing/file")).evaluate(args()));
        assert!(!Exists(PathBuf::from("/non/existing/dir/")).evaluate(args()));
    }

    #[test]
    fn is_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file");
        let _ = std::fs::File::create(file_path.clone());
        assert!(IsFile(file_path).evaluate(args()));
        assert!(!IsFile(dir.path().to_path_buf()).evaluate(args()));
        assert!(!IsFile(PathBuf::from("/non/existing/file")).evaluate(args()));
        assert!(!IsFile(PathBuf::from("/non/existing/dir/")).evaluate(args()));
    }

    #[test]
    fn is_directory() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file");
        let _ = std::fs::File::create(file_path.clone());
        assert!(!IsDirectory(file_path).evaluate(args()));
        assert!(IsDirectory(dir.path().to_path_buf()).evaluate(args()));
        assert!(!IsDirectory(PathBuf::from("/non/existing/file")).evaluate(args()));
        assert!(!IsDirectory(PathBuf::from("/non/existing/dir/")).evaluate(args()));
    }
}
