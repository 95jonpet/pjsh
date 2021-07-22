pub struct Options {
    pub debug_lexing: bool,
    pub debug_parsing: bool,
    pub print_input: bool,
    pub allow_unset_vars: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            debug_lexing: false,
            debug_parsing: false,
            print_input: false,
            allow_unset_vars: true,
        }
    }
}
