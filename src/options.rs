pub struct Options {
    pub debug_lexing: bool,
    pub debug_parsing: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            debug_lexing: false,
            debug_parsing: false,
        }
    }
}
