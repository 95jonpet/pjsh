use std::{
    collections::{HashMap, VecDeque},
    mem::take,
};

use crate::{
    cursor::{Cursor, EOF_CHAR, PS2},
    token::{Expression, Token, Unit},
};

use super::Mode;

pub(crate) struct PosixLexer {
    current_token: String,
    current_units: Vec<Unit>,
    forming_operator: bool,
    operators: HashMap<String, Token>,
    mode: Mode,
    whitespace_chars: Vec<char>,
    cached_tokens: VecDeque<Token>,
}

impl PosixLexer {
    pub(crate) fn new() -> Self {
        let mut operators = HashMap::new();

        // Treat newlines as operators.
        // Both Unix (LF) and Windows (CRLF) are recognized as newlines.
        operators.insert(String::from("\n"), Token::Newline);
        operators.insert(String::from("\r\n"), Token::Newline);

        operators.insert(String::from("|"), Token::Pipe);
        operators.insert(String::from("("), Token::LParen);
        operators.insert(String::from(")"), Token::RParen);
        operators.insert(String::from("<"), Token::Less);
        operators.insert(String::from(">"), Token::Great);
        operators.insert(String::from("&"), Token::And);
        operators.insert(String::from(";"), Token::Semi);
        operators.insert(String::from("&&"), Token::AndIf);
        operators.insert(String::from("||"), Token::OrIf);
        operators.insert(String::from(";;"), Token::DSemi);
        operators.insert(String::from("<<"), Token::DLess);
        operators.insert(String::from(">>"), Token::DGreat);
        operators.insert(String::from("<&"), Token::LessAnd);
        operators.insert(String::from(">&"), Token::GreatAnd);
        operators.insert(String::from("<>"), Token::LessGreat);
        operators.insert(String::from("<<-"), Token::DLessDash);
        operators.insert(String::from(">|"), Token::Clobber);

        Self {
            current_token: String::new(),
            current_units: Vec::new(),
            forming_operator: false,
            mode: Mode::Unquoted,
            operators,
            whitespace_chars: vec![' ', '\t', '\r', '\n'],
            cached_tokens: VecDeque::new(),
        }
    }

    fn delimit_current_token(&mut self, allow_operator: bool) -> Token {
        if self.current_token.is_empty() {
            return Token::Word(take(&mut self.current_units));
        }

        if allow_operator {
            if let Some(token) = self.operators.get(&self.current_token) {
                self.current_token = String::new();

                if !self.current_units.is_empty() {
                    self.cached_tokens.push_back(token.clone());
                    return Token::Word(take(&mut self.current_units));
                } else {
                    return token.clone();
                }
            }
        }

        self.current_units
            .push(self.delimit_unit(&self.current_token));
        self.current_token = String::new();
        Token::Word(take(&mut self.current_units))
    }

    /// Returns `true` if the input string is a prefix, or complete, operator definition.
    fn is_operator_prefix(&self, string: &str) -> bool {
        if string.is_empty() {
            return false;
        }

        self.operators
            .keys()
            .any(|operator| operator.starts_with(string))
    }

    /// Returns `true` if the input character is a prefix, or complete, redirection operator definition.
    fn is_redirection_operator_prefix(&self, ch: char) -> bool {
        return ch == '<' || ch == '>';
    }

    pub(crate) fn next_token(&mut self, cursor: &mut Cursor) -> Token {
        if let Some(cached_token) = self.cached_tokens.pop_front() {
            return cached_token;
        }

        loop {
            let current = cursor.next();
            let mut joined = self.current_token.clone();
            joined.push(current);
            let potential_operator = self.forming_operator || self.current_token.is_empty();
            match current {
                // 1. If the end of input is recognized, the current token (if any) shall be
                // delimited.
                EOF_CHAR if self.mode == Mode::InSingleQuotes => cursor.advance_line(PS2),
                EOF_CHAR if self.mode == Mode::Unquoted => {
                    return self.delimit_token_before_eof(potential_operator);
                }

                // 2. If the previous character was used as part of an operator and the current
                // character is not quoted and can be used with the previous characters to form
                // an operator, it shall be used as part of that (operator) token.
                _ if potential_operator && self.is_operator_prefix(&joined) => {
                    self.forming_operator = true;
                    self.current_token = joined;
                }

                // 3. If the previous character was used as part of an operator and the current
                // character cannot be used with the previous characters to form an operator,
                // the operator containing the previous character shall be delimited.
                ch if self.forming_operator && !self.is_operator_prefix(&joined) => {
                    let operator = self.delimit_operator_token(&self.current_token);

                    if !self.whitespace_chars.contains(&ch) {
                        self.current_token = ch.to_string();
                    } else {
                        self.current_token = String::new();
                    }

                    self.forming_operator = self.is_operator_prefix(&self.current_token);
                    return operator;
                }

                // 4. If the current character is <backslash>, single-quote, or double-quote and
                // it is not quoted, it shall affect quoting for subsequent characters up to
                // the end of the quoted text.
                // During token recognition no substitutions shall be actually performed, and
                // the result token shall contain exactly the characters that appear in the input
                // (except for <newline> joining), unmodified, including any embedded or enclosing
                // quotes or substitution operators, between the <quotation-mark> and the end of
                // the quoted text. The token shall not be delimited by the end of the quoted field.
                // TODO: Implement multiple modes.
                '\'' if self.mode == Mode::Unquoted => {
                    self.mode = Mode::InSingleQuotes;
                    self.forming_operator = false;

                    if !self.current_token.is_empty() {
                        self.current_units
                            .push(self.delimit_unit(&self.current_token));
                    }

                    self.current_token = String::new();
                }
                '\'' if self.mode == Mode::InSingleQuotes => {
                    self.mode = Mode::Unquoted;
                    self.current_units
                        .push(Unit::Literal(take(&mut self.current_token)));
                }
                '"' if self.mode == Mode::Unquoted => {
                    self.mode = Mode::InDoubleQuotes;
                    self.forming_operator = false;

                    if !self.current_token.is_empty() {
                        self.current_units
                            .push(self.delimit_unit(&self.current_token));
                    }

                    self.current_token = String::new();
                }
                '"' if self.mode == Mode::InDoubleQuotes => {
                    self.mode = Mode::Unquoted;

                    if !self.current_token.is_empty() {
                        self.current_units
                            .push(self.delimit_unit(&self.current_token));
                    }

                    self.current_token = String::new();
                }

                // 5. If the current character is an unquoted '$' or '`', the shell shall identify
                // the start of any candidates for parameter expansion (Parameter Expansion),
                // command substitution (Command Substitution), or arithmetic expansion
                // (Arithmetic Expansion) from their introductory unquoted character sequences:
                // '$' or "${", "$(" or '`', and "$((", respectively.
                // The shell shall read sufficient input to determine the end of the unit to be
                // expanded.
                // While processing the characters, if instances of expansions or quoting are found
                // nested within the substitution, the shell shall recursively process them in the
                // manner specified for the construct that is found.
                // The characters found from the beginning of the substitution to its end, allowing
                // for any recursion necessary to recognize embedded constructs, shall be included
                // unmodified in the result token, including any embedded or enclosing substitution
                // operators or quotes. The token shall not be delimited by the end of the
                // substitution.
                // TODO: Handle expansion and substitution syntax.
                '$' if self.mode == Mode::Unquoted || self.mode == Mode::InDoubleQuotes => {
                    if !self.current_token.is_empty() {
                        self.current_units
                            .push(Unit::Literal(take(&mut self.current_token)));
                    }

                    self.current_units.push(self.next_expandable_unit(cursor));
                }

                // 6. If the current character is not quoted and can be used as the first character
                // of a new operator, the current token (if any) shall be delimited.
                // The current character shall be used as the beginning of the next operator token.
                ch if self.mode == Mode::Unquoted
                    && !self.is_operator_prefix(&self.current_token)
                    && self.is_operator_prefix(&ch.to_string()) =>
                {
                    // Allow IO_NUMBER tokens to be found.
                    if self.is_redirection_operator_prefix(ch) {
                        if let Ok(number) = u8::from_str_radix(&self.current_token, 10) {
                            self.forming_operator = true;
                            self.current_token = ch.to_string();
                            return Token::IoNumber(number);
                        }
                    }

                    let token = self.delimit_current_token(potential_operator);
                    self.forming_operator = true;
                    self.current_token = ch.to_string();
                    return token;
                }

                // 7. If the current character is an unquoted <blank>, any token containing the
                // previous character is delimited and the current character shall be discarded.
                ch if self.mode == Mode::Unquoted && self.whitespace_chars.contains(&ch) => {
                    if self.current_token.is_empty() && self.current_units.is_empty() {
                        continue;
                    }

                    if self.forming_operator {
                        self.forming_operator = false;
                        return self.delimit_operator_token(&self.current_token);
                    }

                    return self.delimit_current_token(potential_operator);
                }

                // 9. If the current character is a '#', it and all subsequent characters up to,
                // but excluding, the next <newline> shall be discarded as a comment.
                // The <newline> that ends the line is not considered part of the comment.
                '#' if self.mode == Mode::Unquoted => {
                    while cursor.peek() != &'\n' && cursor.peek() != &EOF_CHAR {
                        cursor.next();
                    }
                }

                // 8. If the previous character was part of a word, the current character shall be
                // appended to that word.
                _ if !self.current_token.is_empty() => {
                    self.current_token = joined;
                }

                // 10. The current character is used as the start of a new word.
                ch => {
                    self.forming_operator = false;
                    self.current_token = ch.to_string();
                }
            }
        }
    }

    /// Reads characters from a [`Cursor`] until a predicate holds.
    /// Returns a [`String`] containing all characters that were passed.
    fn read_until<P>(&self, cursor: &mut Cursor, advance_lines: bool, predicate: P) -> String
    where
        P: Fn(&char) -> bool,
    {
        let can_advance_lines = advance_lines && cursor.is_interactive();
        let mut result = String::new();
        loop {
            match cursor.peek() {
                &EOF_CHAR if can_advance_lines => cursor.advance_line(PS2),
                &EOF_CHAR => break,
                '\\' => {
                    cursor.skip('\\');
                    match cursor.next() {
                        '$' => result.push('$'),
                        '`' => result.push('`'),
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '\n' => (), // Escaped newline.
                        '\r' => {
                            // Escape Windows newline.
                            if cursor.peek() == &'\n' {
                                cursor.next();
                            } else {
                                result.push('\\');
                                result.push('\r');
                            }
                        }
                        ch => {
                            result.push('\\');
                            result.push(ch);
                        }
                    }
                }
                ch if !predicate(ch) => result.push(cursor.next()),
                _ => break,
            }
        }
        result
    }

    /// Lex the next expandable [`Unit`].
    fn next_expandable_unit(&self, cursor: &mut Cursor) -> Unit {
        match cursor.peek() {
            '{' => {
                cursor.skip('{');
                let parameter_separators = ":-=?+}";
                let parameter =
                    self.read_until(cursor, true, |ch| parameter_separators.contains(*ch));

                let unset_or_null = cursor.peek() == &':';
                if unset_or_null {
                    cursor.skip(':');
                }

                match cursor.peek() {
                    &'-' => {
                        cursor.skip('-');
                        let word = self.read_until(cursor, true, |ch| ch == &'}');
                        cursor.skip('}');
                        Unit::Expression(Expression::UseDefaultValues(
                            parameter,
                            word,
                            unset_or_null,
                        ))
                    }
                    &'=' => {
                        cursor.skip('=');
                        let word = self.read_until(cursor, true, |ch| ch == &'}');
                        cursor.skip('}');
                        Unit::Expression(Expression::AssignDefaultValues(
                            parameter,
                            word,
                            unset_or_null,
                        ))
                    }
                    &'?' => {
                        cursor.skip('?');
                        let word = self.read_until(cursor, true, |ch| ch == &'}');
                        cursor.skip('}');
                        Unit::Expression(Expression::IndicateError(parameter, word, unset_or_null))
                    }
                    &'+' => {
                        cursor.skip('+');
                        let word = self.read_until(cursor, true, |ch| ch == &'}');
                        cursor.skip('}');
                        Unit::Expression(Expression::UseAlternativeValue(
                            parameter,
                            word,
                            unset_or_null,
                        ))
                    }
                    &'}' if !unset_or_null => {
                        cursor.skip('}');
                        Unit::Expression(Expression::Parameter(parameter))
                    }
                    ch => todo!("unexpected character '{}'", ch),
                }
            }
            _ => {
                let word = self.read_until(cursor, false, |ch| self.whitespace_chars.contains(ch));
                Unit::Var(word)
            }
        }
    }

    /// Delimits an operator [`Token`].
    fn delimit_operator_token(&self, operator: &str) -> Token {
        self.operators
            .get(operator)
            .expect("the current token should be an operator")
            .to_owned()
    }

    /// Delimits a [`Token`] that exists right before EOF.
    fn delimit_token_before_eof(&mut self, potential_operator: bool) -> Token {
        self.forming_operator = false;

        if self.current_token.is_empty() && self.current_units.is_empty() {
            Token::EOF
        } else {
            self.delimit_current_token(potential_operator)
        }
    }

    fn delimit_unit(&self, current_token: &str) -> Unit {
        // TODO: Remove unreachable code.
        if let Some(_expression) = current_token
            .strip_prefix("${")
            .and_then(|s| s.strip_suffix('}'))
        {
            unreachable!();
            // Unit::Expression(expression.to_string())
        } else if let Some(_var) = current_token.strip_prefix('$') {
            unreachable!();
            // Unit::Var(var.to_string())
        } else {
            Unit::Literal(current_token.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{cursor::PS1, input::InputLines, options::Options};

    use super::*;

    #[test]
    fn it_lexes_words() {
        let mut test_cases = HashMap::new();
        test_cases.insert("word1 word2", vec!["word1", "word2"]);
        test_cases.insert("ls -lah", vec!["ls", "-lah"]);
        test_cases.insert("cat /tmp/tmp_file", vec!["cat", "/tmp/tmp_file"]);
        test_cases.insert("key=value", vec!["key=value"]);

        for (input, words) in test_cases {
            let expected_tokens: Vec<Token> = words
                .iter()
                .map(|word| Token::Word(vec![Unit::Literal(word.to_string())]))
                .collect();
            assert_eq!(
                lex(input),
                expected_tokens,
                "lexing {:?} should yield tokens {:?}",
                input,
                expected_tokens
            )
        }

        assert_eq!(
            lex("ls -lah\n"),
            vec![
                Token::Word(vec![Unit::Literal(String::from("ls"))]),
                Token::Word(vec![Unit::Literal(String::from("-lah"))]),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn it_lexes_io_numbers() {
        assert_eq!(lex("2>"), vec![Token::IoNumber(2), Token::Great]);
    }

    #[test]
    fn it_lexes_operators() {
        let operators = PosixLexer::new().operators;
        for (lexeme, token) in operators.iter() {
            assert_eq!(
                lex(&lexeme),
                vec![token.clone()],
                "lexing {:?} should yield token {:?}",
                lexeme,
                token
            );
        }
    }

    #[test]
    fn it_lexes_operators_mixed_with_words() {
        let mut test_cases = HashMap::new();
        test_cases.insert(
            "word>file",
            vec![
                Token::Word(vec![Unit::Literal(String::from("word"))]),
                Token::Great,
                Token::Word(vec![Unit::Literal(String::from("file"))]),
            ],
        );
        test_cases.insert(
            "echo 1 && echo 2",
            vec![
                Token::Word(vec![Unit::Literal(String::from("echo"))]),
                Token::Word(vec![Unit::Literal(String::from("1"))]),
                Token::AndIf,
                Token::Word(vec![Unit::Literal(String::from("echo"))]),
                Token::Word(vec![Unit::Literal(String::from("2"))]),
            ],
        );

        for (lexeme, expected_tokens) in test_cases {
            assert_eq!(
                lex(&lexeme),
                expected_tokens,
                "lexing {:?} should yield tokens {:?}",
                lexeme,
                expected_tokens
            );
        }
    }

    #[test]
    fn it_lexes_single_quoted_words() {
        assert_eq!(
            lex("echo 'quoted'\n"),
            vec![
                Token::Word(vec![Unit::Literal(String::from("echo"))]),
                Token::Word(vec![Unit::Literal(String::from("quoted"))]),
                Token::Newline,
            ]
        );
        assert_eq!(
            lex("echo 'quoted' unquoted\n"),
            vec![
                Token::Word(vec![Unit::Literal(String::from("echo"))]),
                Token::Word(vec![Unit::Literal(String::from("quoted"))]),
                Token::Word(vec![Unit::Literal(String::from("unquoted"))]),
                Token::Newline,
            ]
        );
        assert_eq!(
            lex("'line 1\nline 2'"),
            vec![Token::Word(vec![Unit::Literal(String::from(
                "line 1\nline 2"
            ))])]
        );
        assert_eq!(
            lex("outside'inside'outside"),
            vec![Token::Word(vec![
                Unit::Literal(String::from("outside")),
                Unit::Literal(String::from("inside")),
                Unit::Literal(String::from("outside")),
            ])]
        );
        assert_eq!(
            lex("'# not a comment'"),
            vec![Token::Word(vec![Unit::Literal(String::from(
                "# not a comment"
            ))])]
        );
    }

    #[test]
    fn it_lexes_variables() {
        let mut test_cases = HashMap::new();
        test_cases.insert("$var", vec!["var"]);

        for (input, words) in test_cases {
            let expected_tokens: Vec<Token> = words
                .iter()
                .map(|word| Token::Word(vec![Unit::Var(word.to_string())]))
                .collect();
            assert_eq!(
                lex(input),
                expected_tokens,
                "lexing {:?} should yield tokens {:?}",
                input,
                expected_tokens
            )
        }
    }

    #[test]
    fn it_lexes_expressions() {
        let mut test_cases = HashMap::new();
        test_cases.insert(
            "${parameter}",
            vec![Expression::Parameter(String::from("parameter"))],
        );
        test_cases.insert(
            "${parameter-word}",
            vec![Expression::UseDefaultValues(
                String::from("parameter"),
                String::from("word"),
                false,
            )],
        );
        test_cases.insert(
            "${parameter:-word}",
            vec![Expression::UseDefaultValues(
                String::from("parameter"),
                String::from("word"),
                true,
            )],
        );
        test_cases.insert(
            "${parameter=word}",
            vec![Expression::AssignDefaultValues(
                String::from("parameter"),
                String::from("word"),
                false,
            )],
        );
        test_cases.insert(
            "${parameter:=word}",
            vec![Expression::AssignDefaultValues(
                String::from("parameter"),
                String::from("word"),
                true,
            )],
        );
        test_cases.insert(
            "${parameter?word}",
            vec![Expression::IndicateError(
                String::from("parameter"),
                String::from("word"),
                false,
            )],
        );
        test_cases.insert(
            "${parameter:?word}",
            vec![Expression::IndicateError(
                String::from("parameter"),
                String::from("word"),
                true,
            )],
        );
        test_cases.insert(
            "${parameter+word}",
            vec![Expression::UseAlternativeValue(
                String::from("parameter"),
                String::from("word"),
                false,
            )],
        );
        test_cases.insert(
            "${parameter:+word}",
            vec![Expression::UseAlternativeValue(
                String::from("parameter"),
                String::from("word"),
                true,
            )],
        );

        for (input, words) in test_cases {
            let expected_tokens: Vec<Token> = words
                .iter()
                .map(|expression| Token::Word(vec![Unit::Expression(expression.clone())]))
                .collect();
            assert_eq!(
                lex(input),
                expected_tokens,
                "lexing {:?} should yield tokens {:?}",
                input,
                expected_tokens
            )
        }

        assert_eq!(
            lex("${a}b"),
            vec![Token::Word(vec![
                Unit::Expression(Expression::Parameter(String::from("a"))),
                Unit::Literal(String::from("b"))
            ])]
        );

        assert_eq!(
            lex("echo \"Hello, ${name:-Mr Smith}\""),
            vec![
                Token::Word(vec![Unit::Literal(String::from("echo"))]),
                Token::Word(vec![
                    Unit::Literal(String::from("Hello, ")),
                    Unit::Expression(Expression::UseDefaultValues(
                        String::from("name"),
                        String::from("Mr Smith"),
                        true,
                    ))
                ]),
            ]
        )
    }

    #[test]
    fn it_skips_comments() {
        assert_eq!(lex("# Comment"), vec![]);
        assert_eq!(lex("# Comment\n"), vec![Token::Newline]);
        assert_eq!(
            lex("# Comment\nword"),
            vec![
                Token::Newline,
                Token::Word(vec![Unit::Literal(String::from("word"))])
            ]
        );
    }

    fn lex(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let options = Rc::new(RefCell::new(Options::default()));
        let mut cursor = Cursor::new(
            InputLines::Single(Some(String::from(input))),
            false,
            options.clone(),
        );
        let mut lexer = PosixLexer::new();
        cursor.advance_line(PS1);

        loop {
            let token = lexer.next_token(&mut cursor);
            if token == Token::EOF {
                break;
            }
            tokens.push(token);
        }

        tokens
    }
}
