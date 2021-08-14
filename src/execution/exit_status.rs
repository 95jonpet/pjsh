/// The exit status for a command.
pub struct ExitStatus {
    code: i32,
}

impl ExitStatus {
    /// Exit status for a command which exited nominally.
    pub const SUCCESS: Self = Self { code: 0 };

    /// Creates a new exit status representation.
    pub fn new(code: i32) -> Self {
        Self { code }
    }

    /// Returns `true` if the exit was successful.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_success_if_code_is_zero() {
        assert_eq!(ExitStatus::SUCCESS.is_success(), true);
        assert_eq!(ExitStatus::new(0).is_success(), true);
        assert_eq!(ExitStatus::new(1).is_success(), false);
    }
}
