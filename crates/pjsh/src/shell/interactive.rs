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

use super::{Shell, ShellInput};

pub struct RustylineShell {
    editor: Editor<ShellHelper>,
    interactive: bool,
}

impl RustylineShell {
    pub fn new(history_file: &std::path::Path) -> Self {
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

        let interactive = atty::is(atty::Stream::Stdin);
        let mut shell = Self {
            editor,
            interactive,
        };

        if interactive {
            shell.load_history_file(history_file);
        }

        shell
    }

    fn load_history_file(&mut self, history_file: &std::path::Path) {
        if !history_file.exists() {
            return;
        }

        if let Err(error) = self.editor.load_history(&history_file) {
            eprintln!("pjsh: Could not load history file: {}", error);
        }
    }
}

impl Shell for RustylineShell {
    fn prompt_line(&mut self, prompt: &str) -> ShellInput {
        // Set a colored prompt from the input. This prompt allows ANSI control sequences to be
        // passed to the terminal.
        self.editor.helper_mut().expect("No helper").colored_prompt = prompt.to_string();

        // The rustyline::Editor::readline method uses a prompt to determine the cursor's position
        // through the prompt's length in characters. This does not work for colored prompts as the
        // ANSI escape codes contribute to the perceived length. Thus, all ANSII escape sequences
        // must be stripped prior to prompting the user for input.
        let prompt_text = strip_ansi_escapes(prompt);
        match self.editor.readline(&prompt_text) {
            Ok(mut line) => {
                line.push('\n');
                ShellInput::Line(line)
            }
            Err(ReadlineError::Interrupted) => ShellInput::Interrupt,
            Err(ReadlineError::Eof) => ShellInput::Logout,
            Err(error) => {
                eprintln!("pjsh: unhandled input: {}", error);
                ShellInput::None
            }
        }
    }

    fn add_history_entry(&mut self, line: &str) {
        self.editor.add_history_entry(line);
    }

    fn is_interactive(&self) -> bool {
        self.interactive
    }

    fn save_history(&mut self, path: &std::path::Path) {
        if let Err(error) = self.editor.append_history(&path) {
            println!("pjsh: Could not write history file: {}", error);
        }
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
