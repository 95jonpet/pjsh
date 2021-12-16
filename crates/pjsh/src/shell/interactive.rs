use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;
use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::{Hinter, HistoryHinter},
    validate::{self, MatchingBracketValidator, Validator},
    Config, Context, Editor,
};
use rustyline_derive::Helper;

use super::Shell;

const USER_HISTORY_FILE_NAME: &str = ".pjsh/history.txt";

pub struct RustylineShell {
    editor: Editor<ShellHelper>,
}

impl RustylineShell {
    pub fn new() -> Self {
        let helper = ShellHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: "$ ".to_owned(),
            validator: MatchingBracketValidator::new(),
        };

        let config = Config::builder().build();
        let mut editor = Editor::with_config(config);
        editor.set_helper(Some(helper));

        let mut shell = Self { editor };

        shell.load_history_file();

        shell
    }

    fn load_history_file(&mut self) {
        if let Some(history_file) = dirs::home_dir().map(|mut path| {
            path.push(USER_HISTORY_FILE_NAME);
            path
        }) {
            if history_file.exists() {
                let _ = self.editor.load_history(&history_file);
            }
        }
    }
}

impl Shell for RustylineShell {
    fn prompt_line(&mut self, prompt: &str) -> Option<String> {
        // Set a colored prompt from the input. This prompt allows ANSI control sequences to be
        // passed to the terminal.
        self.editor.helper_mut().expect("No helper").colored_prompt = prompt.to_string();

        // The rustyline::Editor::readline method uses a prompt to determine the cursor's position
        // through the prompt's length in characters. This does not work for colored prompts as the
        // ANSI escape codes contribute to the perceived length. Thus, all ANSII escape sequences
        // must be stripped prior to prompting the user for input.
        let prompt_text = strip_ansi_escapes(prompt);
        self.editor.readline(&prompt_text).ok().map(|mut line| {
            line.push('\n');
            line
        })
    }

    fn add_history_entry(&mut self, line: &str) {
        self.editor.add_history_entry(line);

        if let Some(history_file) = dirs::home_dir().map(|mut path| {
            path.push(USER_HISTORY_FILE_NAME);
            path
        }) {
            let _ = self.editor.append_history(&history_file);
        }
    }

    fn is_interactive(&self) -> bool {
        true
    }
}

#[derive(Helper)]
struct ShellHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ShellHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Cow::Borrowed(&self.colored_prompt)
        } else {
            Cow::Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned("\x1b[2m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for ShellHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

/// Strips all ANSI control sequences from some text.
fn strip_ansi_escapes(text: &str) -> Cow<str> {
    lazy_static! {
        // This regex was taken from the following page:
        // https://superuser.com/questions/380772/removing-ansi-color-codes-from-text-stream
        static ref RE: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    }
    RE.replace_all(text, "")
}
