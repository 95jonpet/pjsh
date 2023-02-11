use pjsh_core::{command::CommandResult, command::Io};

use crate::status;

/// Prints a [`clap::Error`] message to standard out or standard error depending
/// on the error type.
///
/// Clap returns help messages as errors, so this function handles IO writing
/// accordingly.
///
/// Returns an exit code.
pub fn exit_with_parse_error(io: &mut Io, error: clap::Error) -> CommandResult {
    // Select the proper file descriptor and exit code based on the error type.
    // This is required because printing the help, or version, using flags such
    // as "--help" and "--version" should not be considered an error.
    let (fd, code) = match error.use_stderr() {
        true => (&mut io.stderr, status::BUILTIN_ERROR),
        false => (&mut io.stdout, status::SUCCESS),
    };

    // Write the message as-is in order to avoid issues with duplicate command
    // names and/or inconsistent whitespace.
    let _ = writeln!(fd, "{}", error);
    CommandResult::code(code)
}

/// Constructs a new no-op input/output wrapper for a command.
#[cfg(test)]
pub(crate) fn empty_io() -> Io {
    Io {
        stdin: Box::new(std::io::empty()),
        stdout: Box::new(std::io::sink()),
        stderr: Box::new(std::io::sink()),
    }
}

/// Constructs a new Io instance backed by temporary files.
#[cfg(test)]
pub(crate) fn mock_io() -> (Io, std::fs::File, std::fs::File) {
    use tempfile::tempfile;

    let stdout = tempfile().unwrap();
    let stderr = tempfile().unwrap();
    let io = Io::new(
        Box::new(std::io::empty()),
        Box::new(stdout.try_clone().unwrap()),
        Box::new(stderr.try_clone().unwrap()),
    );
    (io, stdout, stderr)
}

/// Reads the entire contents of a file from start to end.
///
/// Note that this will change the current position in the file.
#[cfg(test)]
pub(crate) fn file_contents(file: &mut std::fs::File) -> String {
    use std::io::{Read, Seek};

    let mut string = String::new();
    let _ = file.rewind();
    let _ = file.read_to_string(&mut string);
    string
}
