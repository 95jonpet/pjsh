use std::io;

/// Input/output wrapper for commands.
///
/// Holds references to the following file descriptors:
///  - Standard input (file descriptor 0).
///  - Standard output (file descriptor 1).
///  - Standard error (file descriptor 2).
///
/// # Examples
///
/// Output can be written using the standard [`write!`] and [`writeln!`] macros.
/// ```
/// use pjsh_core::command::Io;
///
/// let mut io = Io::new(Box::new(std::io::stdin()), Box::new(std::io::stdout()), Box::new(std::io::stderr()));
/// writeln!(io.stdout, "This line is printed to standard output.");
/// writeln!(io.stderr, "This line is printed to standard error.");
/// ```
pub struct Io {
    /// File descriptor for standard input.
    pub stdin: Box<dyn io::Read + Send>,
    /// File descriptor for standard output.
    pub stdout: Box<dyn io::Write + Send>,
    /// File descriptor for standard error.
    pub stderr: Box<dyn io::Write + Send>,
}

impl Io {
    /// Constructs a new input/output wrapper for a command.
    pub fn new(
        stdin: Box<dyn io::Read + Send>,
        stdout: Box<dyn io::Write + Send>,
        stderr: Box<dyn io::Write + Send>,
    ) -> Self {
        Self {
            stdin,
            stdout,
            stderr,
        }
    }
}
