use std::{borrow::Cow, path::PathBuf, sync::Arc};

use parking_lot::Mutex;
use pjsh_complete::Completer;
use pjsh_core::{utils::word_var, Context};
use pjsh_parse::{parse, ParseError};
use rustyline::{
    completion::Pair,
    error::ReadlineError,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::{Hinter, HistoryHinter},
    validate::{self, ValidationResult, Validator},
    CompletionType, Config, Editor,
};
use rustyline_derive::Helper;

use crate::{interpolate, interrupt, Shell, USER_HISTORY_FILE_NAME};

use super::{
    utils::{eval_program, print_error},
    ShellError, ShellResult,
};

pub(crate) enum ShellInput {
    /// A line of input.
    Line(String),

    /// Interrupt the current process.
    Interrupt,

    /// Exit the shell.
    Logout,

    /// No input.
    None,
}

/// An interactive shell that prompts the user from input.
///
/// Reads input from stdin.
pub struct InteractiveShell {
    /// Rustyline editor.
    editor: Editor<ShellHelper>,
}

impl InteractiveShell {
    /// Constructs a new interactive shell.
    pub fn new(context: Arc<Mutex<Context>>, completer: Arc<Mutex<Completer>>) -> Self {
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
        let mut editor = Editor::with_config(config).expect("terminal editor should be configured");
        editor.set_helper(Some(helper));

        Self { editor }
    }

    /// Returns a prompted line of input.
    fn prompt_line(&mut self, prompt: &str) -> ShellInput {
        // Set a colored prompt from the input.
        // This prompt allows ANSI control sequences to be passed to the terminal.
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
}

impl Shell for InteractiveShell {
    fn init(&mut self) -> ShellResult<()> {
        let history_file = history_file_path();
        if history_file.exists() {
            self.editor
                .load_history(&history_file)
                .map_err(|err| ShellError::Error(err.to_string()))?;
        }

        Ok(())
    }

    fn run(&mut self, context: Arc<Mutex<Context>>) -> ShellResult<()> {
        'main: loop {
            let (ps1, ps2) = get_prompts(Arc::clone(&context));
            print_exited_child_processes(&mut context.lock());

            let mut line = match self.prompt_line(&ps1) {
                ShellInput::Line(line) => line,
                ShellInput::Interrupt => {
                    interrupt(&mut context.lock());
                    continue;
                }
                ShellInput::Logout => {
                    eprintln!("pjsh: logout");
                    break 'main;
                }
                ShellInput::None => break,
            };

            // Repeatedly ask for lines of input until a valid program can be executed.
            loop {
                let aliases = context.lock().aliases.clone();
                match parse(&line, &aliases) {
                    // If a valid program can be parsed from the buffer, execute it.
                    Ok(program) => {
                        self.editor.add_history_entry(line.trim());
                        eval_program(&program, &mut context.lock(), print_error)?;
                        break;
                    }

                    // If more input is required, prompt for more input and loop again.
                    // The next line of input will be appended to the buffer and parsed.
                    Err(ParseError::IncompleteSequence | ParseError::UnexpectedEof) => {
                        match self.prompt_line(&ps2) {
                            ShellInput::Line(next_line) => line.push_str(&next_line),
                            ShellInput::Interrupt => {
                                interrupt(&mut context.lock());
                                continue 'main;
                            }
                            ShellInput::Logout => {
                                eprintln!("pjsh: logout");
                                break 'main;
                            }
                            ShellInput::None => break,
                        };
                    }

                    // Unrecoverable error.
                    Err(error) => {
                        eprintln!("pjsh: parse error: {}", error);
                    }
                }
            }
        }

        Ok(())
    }

    fn exit(mut self) -> ShellResult<()> {
        let history_file = history_file_path();
        if let Some(parent) = history_file.parent() {
            std::fs::create_dir_all(parent).map_err(|err| ShellError::Error(err.to_string()))?;
        }

        self.editor
            .append_history(&history_file)
            .map_err(|err| ShellError::Error(err.to_string()))?;

        Ok(())
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

/// Get interpolated PS1 and PS2 prompts from a context.
fn get_prompts(context: Arc<Mutex<Context>>) -> (String, String) {
    let raw_ps1 = word_var(&context.lock(), "PS1")
        .unwrap_or("\\$ ")
        .to_owned();
    let raw_ps2 = word_var(&context.lock(), "PS2")
        .unwrap_or("\\> ")
        .to_owned();

    let ps1 = interpolate(&raw_ps1, Arc::clone(&context));
    let ps2 = interpolate(&raw_ps2, Arc::clone(&context));

    (ps1, ps2)
}

/// Returns a path to the current user's shell history file.
fn history_file_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(USER_HISTORY_FILE_NAME);
    path
}

/// Prints process IDs (PIDs) to stderr for each child process that is managed by the shell, and
/// that have exited since last checking.
fn print_exited_child_processes(context: &mut Context) {
    let mut host = context.host.lock();
    for pid in host.take_exited_child_processes() {
        eprintln!("pjsh: PID {pid} exited");
    }
}
