use crate::Word;

/// A redirect causes writing to a file descriptor to affect another file descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    /// File descriptor to redirect from.
    pub source: FileDescriptor,

    /// File descriptor to redirect to.
    pub target: FileDescriptor,

    /// Redirection mode to use.
    pub mode: RedirectMode,
}

impl Redirect {
    /// Constructs a new redirect from one file descriptor to another.
    pub fn new(source: FileDescriptor, target: FileDescriptor, mode: RedirectMode) -> Self {
        Self {
            source,
            target,
            mode,
        }
    }
}

/// The mode to use when redirecting file descriptors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectMode {
    /// Write to the target file descriptor, replacing any previous data.
    Write,

    /// Append to the target file descriptor.
    ///
    /// Any existing data is not replaced when using this mode.
    Append,
}

/// A file descriptor represents a data source and/or sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileDescriptor {
    /// A numbered file descriptor.
    ///
    /// File descriptor 0 represents the standard input, from which input is
    /// normally read.
    ///
    /// File descriptor 1 represents the standard output, to which regular output is
    /// normally written.
    ///
    /// File descriptor 2 represents the standard error output, to which error
    /// output is normally written.
    Number(usize),

    /// A file to read data from or write data to.
    File(Word),
}
