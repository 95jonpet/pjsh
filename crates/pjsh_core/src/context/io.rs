use std::{fs::File, path::PathBuf};

/// A file descriptor is a source, and/or, target for IO operations and redirections within a shell.
#[derive(Debug)]
pub enum FileDescriptor {
    /// Handle for [`std::io::stdin()`].
    Stdin,

    /// Handle for [`std::io::stdout()`].
    Stdout,

    /// Handle for [`std::io::stderr()`].
    Stderr,

    /// Handle for piped input/output.
    Pipe,

    /// A file handle to an opened file.
    FileHandle(File),

    /// A file path. Can be used for reading and/or writing.
    ///
    /// Converted to a [`FileDescriptor::FileHandle(File)`] on use.
    File(PathBuf),

    /// A file path for appending data to. Can only be used for writing.
    ///
    /// Converted to a [`FileDescriptor::FileHandle(File)`] on use.
    AppendFile(PathBuf),
}

impl Clone for FileDescriptor {
    fn clone(&self) -> Self {
        match self {
            Self::FileHandle(file) => Self::FileHandle(file.try_clone().unwrap()),
            file_descriptor => file_descriptor.clone(),
        }
    }
}
