pub struct ExitStatus {
    code: i32,
}

impl ExitStatus {
    pub const SUCCESS: Self = Self { code: 0 };
    pub const COMMAND_NOT_FOUND: Self = Self { code: 127 };
    pub const COMMAND_NOT_EXECUTABLE: Self = Self { code: 126 };

    pub fn new(code: i32) -> Self {
        Self { code }
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        self.code == 0
    }
}
