/// Return all word spans in a text input.
pub(crate) fn separate_input(text: &str) -> Vec<(&str, usize, usize)> {
    let mut words = Vec::new();
    let mut quotes = Vec::with_capacity(16);
    let mut start = None;
    let mut end = 0;

    for (pos, ch) in text.char_indices() {
        end += ch.len_utf8();

        // Skip unquoted whitespace.
        if ch.is_whitespace() && quotes.is_empty() && start.is_none() {
            continue;
        }

        // End words after encountering the outermost closing quote.
        if &ch == quotes.last().unwrap_or(&char::REPLACEMENT_CHARACTER) {
            quotes.pop();
            if quotes.is_empty() {
                let span = (start.take().unwrap_or(0), end); // Include end quote.
                words.push((&text[span.0..span.1], span.0, span.1));
            }
            continue;
        }

        // Start words on first non-whitespace.
        if start.is_none() {
            start = Some(pos);
        }

        // Start words when encountering an opening quote.
        if matches!(ch, '"' | '\'' | '`') {
            quotes.push(ch);
            start = start.or(Some(pos));
            continue;
        }

        // End words when encountering whitesapces and not in quotes.
        if ch.is_whitespace() && quotes.is_empty() {
            let span = (start.take().unwrap_or(0), pos); // Exclude whitespace.
            words.push((&text[span.0..span.1], span.0, span.1));
            continue;
        }
    }

    // Push the final incomplete word.
    if let Some(start) = start {
        if start != end {
            let span = (start, end);
            words.push((&text[span.0..span.1], span.0, span.1));
        }
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<&str> {
        separate_input(text).iter().map(|part| part.0).collect()
    }

    #[test]
    fn get_input_words_from_empty_input() {
        assert_eq!(words(""), Vec::<&str>::default());
    }

    #[test]
    fn get_input_words_separated_by_whitespace() {
        assert_eq!(
            words("first\tsecond third"),
            vec!["first", "second", "third"]
        );
    }

    #[test]
    fn get_input_words_with_quotes() {
        assert_eq!(
            words(r#"first 'still a "word"' second"#),
            vec!["first", r#"'still a "word"'"#, "second"]
        );
        assert_eq!(words(r#"'a'"b"c"#), vec!["'a'", r#""b""#, "c"]);
    }
}
