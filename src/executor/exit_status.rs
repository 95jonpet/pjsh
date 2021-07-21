pub struct ExitStatus {
    code: i32,
}

impl ExitStatus {
    pub const SUCCESS: Self = Self { code: 0 };

    pub fn new(code: i32) -> Self {
        Self { code }
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        self.code == 0
    }
}
