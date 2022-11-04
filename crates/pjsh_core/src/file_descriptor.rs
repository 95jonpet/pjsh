use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
    process::Stdio,
};

use os_pipe::{PipeReader, PipeWriter};

/// Index for the stdin file descriptor.
pub const FD_STDIN: usize = 0;

/// Index for the stdout file descriptor.
pub const FD_STDOUT: usize = 1;

/// Index for the stderr file descriptor.
pub const FD_STDERR: usize = 2;

/// File descriptor-related errors.
pub enum FileDescriptorError {
    UnusableForOutput,
    UnusableForInput,
    FileNotReadable(PathBuf, io::Error),
    FileNotWritable(PathBuf, io::Error),
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
    /// Creates a clone of the file descriptor.
    pub fn try_clone(&self) -> std::io::Result<Self> {
        match self {
            FileDescriptor::Stdin => Ok(FileDescriptor::Stdin),
            FileDescriptor::Stdout => Ok(FileDescriptor::Stdout),
            FileDescriptor::Stderr => Ok(FileDescriptor::Stderr),
            FileDescriptor::Pipe((reader, writer)) => Ok(FileDescriptor::Pipe((
                reader.try_clone()?,
                writer.try_clone()?,
            ))),
            FileDescriptor::FileHandle(file) => Ok(FileDescriptor::FileHandle(file.try_clone()?)),
            FileDescriptor::File(path) => Ok(FileDescriptor::File(path.clone())),
            FileDescriptor::AppendFile(path) => Ok(FileDescriptor::AppendFile(path.clone())),
        }
    }

    /// Returns a [`Stdio`] for writing to.
    pub fn output(&mut self) -> Result<Stdio, FileDescriptorError> {
        match self {
            FileDescriptor::Stdin => Err(FileDescriptorError::UnusableForOutput),
            FileDescriptor::Stdout => Ok(Stdio::inherit()),
            FileDescriptor::Stderr => Ok(Stdio::inherit()),
            FileDescriptor::Pipe((_, writer)) => Ok(Stdio::from(writer.try_clone().unwrap())),
            FileDescriptor::FileHandle(file) => Ok(Stdio::from(file.try_clone().unwrap())),
            FileDescriptor::File(path) => match File::create(&path) {
                Ok(file) => {
                    *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                    Ok(Stdio::from(file))
                }
                Err(error) => Err(FileDescriptorError::FileNotWritable(path.clone(), error)),
            },
            FileDescriptor::AppendFile(path) => {
                match OpenOptions::new().append(true).create(true).open(&path) {
                    Ok(file) => {
                        *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                        Ok(Stdio::from(file))
                    }
                    Err(error) => Err(FileDescriptorError::FileNotWritable(path.clone(), error)),
                }
            }
        }
    }

    /// Returns a [`Stdio`] for reading from.
    pub fn input(&mut self) -> Result<Stdio, FileDescriptorError> {
        match self {
            FileDescriptor::Stdin => Ok(Stdio::inherit()),
            FileDescriptor::Stdout => Err(FileDescriptorError::UnusableForInput),
            FileDescriptor::Stderr => Err(FileDescriptorError::UnusableForInput),
            FileDescriptor::Pipe((reader, _)) => Ok(Stdio::from(reader.try_clone().unwrap())),
            FileDescriptor::File(path) => match File::open(&path) {
                Ok(file) => {
                    *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                    Ok(Stdio::from(file))
                }
                Err(error) => Err(FileDescriptorError::FileNotReadable(path.clone(), error)),
            },
            FileDescriptor::AppendFile(_) => unreachable!(),
            _ => self.output(),
        }
    }

    /// Returns a reader for the file descriptor.
    pub fn reader(&mut self) -> Result<Box<dyn Read + Send>, FileDescriptorError> {
        match self {
            FileDescriptor::Stdin => Ok(Box::new(io::stdin())),
            FileDescriptor::Stdout => Err(FileDescriptorError::UnusableForInput),
            FileDescriptor::Stderr => Err(FileDescriptorError::UnusableForInput),
            FileDescriptor::Pipe((reader, _)) => Ok(Box::new(reader.try_clone().unwrap())),
            FileDescriptor::File(path) => match File::open(&path) {
                Ok(file) => {
                    *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                    Ok(Box::new(file))
                }
                Err(error) => Err(FileDescriptorError::FileNotReadable(path.clone(), error)),
            },
            FileDescriptor::FileHandle(file) => Ok(Box::new(file.try_clone().unwrap())),
            FileDescriptor::AppendFile(_) => unreachable!(),
        }
    }

    /// Returns a writer for the file descriptor.
    pub fn writer(&mut self) -> Result<Box<dyn Write + Send>, FileDescriptorError> {
        match self {
            FileDescriptor::Stdin => Err(FileDescriptorError::UnusableForOutput),
            FileDescriptor::Stdout => Ok(Box::new(io::stdout())),
            FileDescriptor::Stderr => Ok(Box::new(io::stderr())),
            FileDescriptor::Pipe((_, writer)) => Ok(Box::new(writer.try_clone().unwrap())),
            FileDescriptor::FileHandle(file) => Ok(Box::new(file.try_clone().unwrap())),
            FileDescriptor::File(path) => match File::create(&path) {
                Ok(file) => {
                    *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                    Ok(Box::new(file))
                }
                Err(error) => Err(FileDescriptorError::FileNotWritable(path.clone(), error)),
            },
            FileDescriptor::AppendFile(path) => {
                match OpenOptions::new().append(true).create(true).open(&path) {
                    Ok(file) => {
                        *self = FileDescriptor::FileHandle(file.try_clone().unwrap());
                        Ok(Box::new(file))
                    }
                    Err(error) => Err(FileDescriptorError::FileNotWritable(path.clone(), error)),
                }
            }
        }
    }
}
