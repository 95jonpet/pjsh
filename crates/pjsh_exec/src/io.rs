use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
    process::{ChildStdout, Stdio},
};

use os_pipe::{PipeReader, PipeWriter};
use pjsh_core::utils::path_to_string;

use crate::error::ExecError;

/// Index for the stdin file descriptor.
pub(crate) const FD_STDIN: usize = 0;

/// Index for the stdout file descriptor.
pub(crate) const FD_STDOUT: usize = 1;

/// Index for the stderr file descriptor.
pub(crate) const FD_STDERR: usize = 2;

pub enum Input {
    Piped(ChildStdout),
    Value(String),
    Inherit,
}

/// A file descriptor is a source, and/or, target for IO operations and redirections within a shell.
#[derive(Debug)]
pub enum FileDescriptor {
    /// Handle for [`std::io::stdin()`].
    Stdin,
    /// Handle for [`std::io::stdout()`].
    Stdout,
    /// Handle for [`std::io::stderr()`].
    Stderr,
    /// A pipe with a [`PipeReader`] output and a [`PipeWriter`] input.
    Pipe((PipeReader, PipeWriter)),
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

impl FileDescriptor {
    /// Returns a [`Stdio`] for writing to.
    pub fn output(&mut self) -> Result<Stdio, ExecError> {
        match self {
            FileDescriptor::Stdin => {
                Err(ExecError::Message("stdin cannot be written to".to_string()))
            }
            FileDescriptor::Stdout => Ok(Stdio::inherit()),
            FileDescriptor::Stderr => Ok(Stdio::inherit()),
            FileDescriptor::Pipe((_, writer)) => Ok(Stdio::from(writer.try_clone().unwrap())),
            FileDescriptor::FileHandle(file) => Ok(Stdio::from(file.try_clone().unwrap())),
            FileDescriptor::File(path) => match File::create(&path) {
                Ok(file) => {
                    *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                    Ok(Stdio::from(file))
                }
                Err(error) => Err(ExecError::Message(format!(
                    "could not open file '{}' for writing: {}",
                    path_to_string(&path),
                    error
                ))),
            },
            FileDescriptor::AppendFile(path) => {
                match OpenOptions::new().append(true).create(true).open(&path) {
                    Ok(file) => {
                        *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                        Ok(Stdio::from(file))
                    }
                    Err(error) => Err(ExecError::Message(format!(
                        "could not open file '{}' for writing: {}",
                        path_to_string(&path),
                        error
                    ))),
                }
            }
        }
    }

    /// Returns a [`Stdio`] for reading from.
    pub fn input(&mut self) -> Result<Stdio, ExecError> {
        match self {
            FileDescriptor::Stdin => Ok(Stdio::inherit()),
            FileDescriptor::Stdout => Err(ExecError::Message("stdout cannot be read".to_string())),
            FileDescriptor::Stderr => Err(ExecError::Message("stderr cannot be read".to_string())),
            FileDescriptor::Pipe((reader, _)) => Ok(Stdio::from(reader.try_clone().unwrap())),
            FileDescriptor::File(path) => match File::open(&path) {
                Ok(file) => {
                    *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                    Ok(Stdio::from(file))
                }
                Err(error) => Err(ExecError::Message(format!(
                    "could not open file '{}' for reading: {}",
                    path_to_string(&path),
                    error
                ))),
            },
            FileDescriptor::AppendFile(_) => unreachable!(),
            _ => self.output(),
        }
    }

    /// Returns a reader for the file descriptor.
    pub fn reader(&mut self) -> Result<Box<dyn Read + Send>, ExecError> {
        match self {
            FileDescriptor::Stdin => Ok(Box::new(io::stdin())),
            FileDescriptor::Stdout => Err(ExecError::Message("stdout cannot be read".to_string())),
            FileDescriptor::Stderr => Err(ExecError::Message("stderr cannot be read".to_string())),
            FileDescriptor::Pipe((reader, _)) => Ok(Box::new(reader.try_clone().unwrap())),
            FileDescriptor::File(path) => match File::open(&path) {
                Ok(file) => {
                    *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                    Ok(Box::new(file))
                }
                Err(error) => Err(ExecError::Message(format!(
                    "could not open file '{}' for reading: {}",
                    path_to_string(&path),
                    error
                ))),
            },
            FileDescriptor::FileHandle(file) => Ok(Box::new(file.try_clone().unwrap())),
            FileDescriptor::AppendFile(_) => unreachable!(),
        }
    }

    /// Returns a writer for the file descriptor.
    pub fn writer(&mut self) -> Result<Box<dyn Write + Send>, ExecError> {
        match self {
            FileDescriptor::Stdin => {
                Err(ExecError::Message("stdin cannot be written to".to_string()))
            }
            FileDescriptor::Stdout => Ok(Box::new(io::stdout())),
            FileDescriptor::Stderr => Ok(Box::new(io::stderr())),
            FileDescriptor::Pipe((_, writer)) => Ok(Box::new(writer.try_clone().unwrap())),
            FileDescriptor::FileHandle(file) => Ok(Box::new(file.try_clone().unwrap())),
            FileDescriptor::File(path) => match File::create(&path) {
                Ok(file) => {
                    *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                    Ok(Box::new(file))
                }
                Err(error) => Err(ExecError::Message(format!(
                    "could not open file '{}' for writing: {}",
                    path_to_string(&path),
                    error
                ))),
            },
            FileDescriptor::AppendFile(path) => {
                match OpenOptions::new().append(true).create(true).open(&path) {
                    Ok(file) => {
                        *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                        Ok(Box::new(file))
                    }
                    Err(error) => Err(ExecError::Message(format!(
                        "could not open file '{}' for writing: {}",
                        path_to_string(&path),
                        error
                    ))),
                }
            }
        }
    }
}

/// A collection of numbered [`FileDescriptor`] instances.
#[derive(Debug)]
pub struct FileDescriptors {
    /// Numbered file descriptors.
    fds: HashMap<usize, FileDescriptor>,
}

impl FileDescriptors {
    /// Creates a new collection of file descriptors with inherited stdin, stdout, and stderr.
    pub fn new() -> Self {
        let mut fds = HashMap::new();

        fds.insert(FD_STDIN, FileDescriptor::Stdin);
        fds.insert(FD_STDOUT, FileDescriptor::Stdout);
        fds.insert(FD_STDERR, FileDescriptor::Stderr);

        Self { fds }
    }

    /// Returns and removes the file descriptor with index `k`.
    pub fn take(&mut self, k: &usize) -> Option<FileDescriptor> {
        self.fds.remove(k)
    }

    /// Returns a [`Stdio`] for writing to the file descriptor with index `k`.
    ///
    /// Returns `None` if no such file descriptor exists.
    pub fn output(&mut self, k: &usize) -> Option<Result<Stdio, ExecError>> {
        self.fds.get_mut(k).map(FileDescriptor::output)
    }

    /// Returns a [`Stdio`] for reading from the file descriptor with index `k`.
    ///
    /// Returns `None` if no such file descriptor exists.
    pub fn input(&mut self, k: &usize) -> Option<Result<Stdio, ExecError>> {
        self.fds.get_mut(k).map(FileDescriptor::input)
    }

    /// Returns a [`Write`] for writing to the file descriptor with index `k`.
    ///
    /// Returns `None` if no such file descriptor exists.
    pub fn writer(&mut self, k: &usize) -> Option<Result<Box<dyn Write + Send>, ExecError>> {
        self.fds.get_mut(k).map(FileDescriptor::writer)
    }

    /// Returns a [`Read`] for reading from the file descriptor with index `k`.
    ///
    /// Returns `None` if no such file descriptor exists.
    pub fn reader(&mut self, k: &usize) -> Option<Result<Box<dyn Read + Send>, ExecError>> {
        self.fds.get_mut(k).map(FileDescriptor::reader)
    }

    /// Updates file descriptor `k`.
    ///
    /// Any previous file descriptor with the same index is dropped.
    pub fn set(&mut self, k: usize, fd: FileDescriptor) {
        self.fds.insert(k, fd);
    }
}
