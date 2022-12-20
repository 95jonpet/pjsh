use pjsh_parse::{is_whitespace, Span};

/// Return all word spans in a text input.
pub fn input_words(text: &str) -> Vec<(&str, Span)> {
    let mut words = Vec::new();
    let mut quotes = Vec::with_capacity(16);
    let mut start = None;
    let mut end = 0;

    for (pos, ch) in text.char_indices() {
        end += ch.len_utf8();

        // Skip unquoted whitespace.
        if is_whitespace(ch) && quotes.is_empty() && start.is_none() {
            continue;
        }

        // End words after encountering the outermost closing quote.
        if &ch == quotes.last().unwrap_or(&char::REPLACEMENT_CHARACTER) {
            quotes.pop();
            if quotes.is_empty() {
                let span = Span::new(start.take().unwrap_or(0), end); // Include end quote.
                words.push((&text[span.start..span.end], span));
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
        if is_whitespace(ch) && quotes.is_empty() {
            let span = Span::new(start.take().unwrap_or(0), pos); // Exclude whitespace.
            words.push((&text[span.start..span.end], span));
            continue;
        }
    }

    // Push the final incomplete word.
    if let Some(start) = start {
        if start != end {
            let span = Span::new(start, end);
            words.push((&text[span.start..span.end], span));
        }
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<&str> {
        input_words(text).iter().map(|(word, _)| *word).collect()
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
