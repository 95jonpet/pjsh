use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;

/// Strips all ANSI control sequences from some text.
pub fn strip_ansi_escapes(text: &str) -> Cow<str> {
    lazy_static! {
        // This regex was taken from the following page:
        // https://superuser.com/questions/380772/removing-ansi-color-codes-from-text-stream
        static ref RE: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    }
    RE.replace_all(text, "")
}
