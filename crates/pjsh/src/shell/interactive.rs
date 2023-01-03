use std::{borrow::Cow, sync::Arc};

use parking_lot::Mutex;
use pjsh_complete::Completer;
use pjsh_core::{utils::word_var, Context};
use rustyline::{
    completion::Pair,
    error::ReadlineError,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::{Hinter, HistoryHinter},
    validate::{self, ValidationResult, Validator},
    CompletionType, Config, Editor,
};
use rustyline_derive::Helper;

use crate::shell::{Shell, ShellInput};

/// An interactive shell backed by [`rustyline`].
pub struct RustylineShell {
    /// Rustyline editor.
    editor: Editor<ShellHelper>,
}

impl RustylineShell {
    /// Constructs a new interactive shell backed by rustyline.
    ///
    /// Shell command history is read from a file.
    pub fn new(
        history_file: &std::path::Path,
        context: Arc<Mutex<Context>>,
        completer: Arc<Mutex<Completer>>,
    ) -> Self {
        let completion_type = match word_var(&context.lock(), "PJSH_COMPLETION_TYPE") {
            Some("circular") => CompletionType::Circular,
            Some("list") | None => CompletionType::List,
            Some(other) => {
                eprintln!("pjsh: Invalid completion type: {other}");
                CompletionType::List
            }
        };

        let helper = ShellHelper {
            context,
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            completer,
            colored_prompt: "$ ".to_owned(),
        };

        let config = Config::builder().completion_type(completion_type).build();
        let mut editor = Editor::with_config(config).expect("configure editor");
        editor.set_helper(Some(helper));

        let mut shell = Self { editor };
        shell.load_history_file(history_file);

        shell
    }

    /// Loads shell command history from a file.
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

        match self.editor.readline(prompt) {
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
        true
    }

    fn save_history(&mut self, path: &std::path::Path) {
        if let Some(parent) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                println!("pjsh: Could not write history file: {error}");
                return;
            }
        }

        if let Err(error) = self.editor.append_history(&path) {
            println!("pjsh: Could not write history file: {error}");
        }
    }
}

/// Rustyline shell helper for enhancing the user experience.
#[derive(Helper)]
struct ShellHelper {
    /// Shell execution context.
    context: Arc<Mutex<Context>>,

    /// Text color highlighter.
    highlighter: MatchingBracketHighlighter,

    /// Suggestion hinter.
    hinter: HistoryHinter,

    /// Line completion provider.
    completer: Arc<Mutex<Completer>>,

    /// Colored shell prompt optionally containing ANSI control sequences.
    colored_prompt: String,
}

impl rustyline::completion::Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let completion = (self.completer.lock()).complete_line(line, pos, &self.context.lock());
        let pairs = completion
            .replacements
            .into_iter()
            .map(|replacement| Pair {
                display: replacement.display,
                replacement: replacement.content,
            })
            .collect();
        Ok((completion.line_pos, pairs))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
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
    fn validate(&self, _: &mut validate::ValidationContext) -> rustyline::Result<ValidationResult> {
        // The lexer/parser is responsible for validating input. Thus, the interactive shell should
        // consider all input valid at this point.
        rustyline::Result::Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}
