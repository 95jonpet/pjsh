/// Returns `true` if a character should be considered a newline.
pub fn is_newline(c: &char) -> bool {
    matches!(
        c,
        '\u{000A}'   // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0085}' // next line
        | '\u{2028}' // line separator
        | '\u{2029}' // paragraph separator
    )
}

/// Returns `true` if a character is allowed in a variable name.
pub fn is_variable_char(c: &char) -> bool {
    c.is_alphanumeric() || c == &'_'
}

/// Returns `true` if a character should be considered whitespace.
pub fn is_whitespace(c: &char) -> bool {
    matches!(
        c,
        // ASCII
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}
