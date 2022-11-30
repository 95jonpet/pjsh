use std::{
    collections::VecDeque,
    env::temp_dir,
    io::{BufReader, Read, Seek},
    path::PathBuf,
};

use dirs::home_dir;
use pjsh_ast::{Function, InterpolationUnit, Program, Word};
use pjsh_core::{utils::path_to_string, Context, FileDescriptor, FD_STDOUT};
use rand::Rng;
use tempfile::tempfile;

use crate::{
    call::call_function,
    error::{EvalError, EvalResult},
    execute_subshell,
};

/// Expands words.
pub fn expand_words(words: &[Word], context: &Context) -> EvalResult<Vec<String>> {
    if words.is_empty() {
        return Ok(Vec::new());
    }

    let is_aliasable = matches!(words[0], Word::Literal(_) | Word::Variable(_));
    let mut words = interpolate_words(words, context)?;

    if is_aliasable {}

    Ok(Vec::from(std::mem::take(&mut words.make_contiguous())))
}

/// Interpolates words.
fn interpolate_words(words: &[Word], context: &Context) -> EvalResult<VecDeque<String>> {
    let mut interpolated_words = VecDeque::with_capacity(words.len());
    for word in words {
        let is_globbable = matches!(word, Word::Literal(_));
        let word = interpolate_word(word, context)?;

        if is_globbable {
            interpolated_words.extend(expand_globs(word, context));
        } else {
            interpolated_words.push_back(word);
        }
    }
    Ok(interpolated_words)
}

/// Expands globs.
fn expand_globs(mut word: String, context: &Context) -> VecDeque<String> {
    expand_tilde(&mut word, context);
    expand_asterisk(word)
}

/// Expands asterisks (`*`).
fn expand_asterisk(word: String) -> VecDeque<String> {
    let mut words = VecDeque::with_capacity(1);

    if let Some(index) = word.find('*') {
        let head = &word[0..index];
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        path.push(head);

        // Cannot expand glob, keep the asterisk.
        if !path.exists() {
            words.push_back(word);
            return words;
        }

        let mut globbed = Vec::new();
        for entry in path.read_dir().unwrap() {
            let file_name = entry.unwrap().file_name().to_string_lossy().to_string();

            if file_name.starts_with('.') {
                continue;
            }

            let mut file = word.clone();
            file.replace_range(index..index + 1, &file_name);
            globbed.push(file);
        }
        globbed.sort();
        words.extend(globbed);
    } else {
        words.push_back(word);
    }

    words
}

/// Expands the tilde (`~`) symbol.
fn expand_tilde(word: &mut String, context: &Context) {
    if word.starts_with('~') {
        let home = context.get_var("HOME").unwrap_or("/");

        // Replace the word.
        *word = word.replacen('~', home, 1);
    }
}

/// Interpolates a word.
pub fn interpolate_word(word: &Word, context: &Context) -> EvalResult<String> {
    match word {
        Word::Literal(literal) => Ok(literal.clone()),
        Word::Quoted(quoted) => Ok(quoted.clone()),
        Word::Variable(variable_name) => interpolate_variable(variable_name, context),
        Word::Subshell(subshell) => interpolate_subshell(subshell, context),
        Word::ProcessSubstitution(process) => substitute_process(process, context),
        Word::Interpolation(units) => interpolate_units(units, context),
    }
}

/// Interpolates word units.
fn interpolate_units(units: &[InterpolationUnit], context: &Context) -> EvalResult<String> {
    let mut output = String::new();

    for unit in units {
        match unit {
            pjsh_ast::InterpolationUnit::Literal(literal) => output.push_str(literal),
            pjsh_ast::InterpolationUnit::Unicode(ch) => output.push(ch.to_owned()),
            pjsh_ast::InterpolationUnit::Variable(name) => {
                output.push_str(&interpolate_variable(name, context)?);
            }
            pjsh_ast::InterpolationUnit::Subshell(subshell) => {
                output.push_str(&interpolate_subshell(subshell, context)?);
            }
        }
    }

    Ok(output)
}

/// Interpolates a subshell.
fn interpolate_subshell(subshell: &Program, context: &Context) -> EvalResult<String> {
    interpolate(context, |context| execute_subshell(subshell, context))
}

/// Interpolates a function call.
pub fn interpolate_function_call(
    function: &Function,
    args: &[String],
    context: &Context,
) -> EvalResult<String> {
    interpolate(context, |mut context| {
        call_function(function, args, &mut context)
    })
}

/// Returns the interpolated stdout of a function.
fn interpolate(context: &Context, func: impl Fn(Context) -> EvalResult<()>) -> EvalResult<String> {
    let mut inner_context = context.try_clone().map_err(EvalError::ContextCloneFailed)?;

    let stdout = tempfile().map_err(EvalError::IoError)?;
    let stdout_fd = FileDescriptor::FileHandle(stdout.try_clone().map_err(EvalError::IoError)?);
    inner_context.set_file_descriptor(FD_STDOUT, stdout_fd);

    func(inner_context)?;

    let read_file = |mut file: std::fs::File| {
        let _ = file.seek(std::io::SeekFrom::Start(0));
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        let _ = buf_reader.read_to_string(&mut contents);

        // Trim any final newline that are normally used to separate the shell output and prompt.
        if let Some('\n') = contents.chars().last() {
            contents.truncate(contents.len() - 1);
            if let Some('\r') = contents.chars().last() {
                contents.truncate(contents.len() - 1);
            }
        }

        contents
    };

    Ok(read_file(stdout))
}

/// Interpolates a variable within a context.
fn interpolate_variable(variable_name: &str, context: &Context) -> EvalResult<String> {
    match variable_name {
        "$" => Ok(std::process::id().to_string()),
        "?" => Ok(context.last_exit().to_string()),
        "HOME" => home_dir().map_or_else(
            || Err(EvalError::UndefinedVariable("HOME".to_owned())),
            |path| Ok(path_to_string(path)),
        ),
        "PWD" => std::env::current_dir().map_or_else(
            |err| Err(EvalError::IoError(err)),
            |path| Ok(path_to_string(path)),
        ),
        "SHELL" => std::env::current_exe().map_or_else(
            |err| Err(EvalError::IoError(err)),
            |path| Ok(path_to_string(path)),
        ),
        _ => match context.get_var(variable_name) {
            Some(value) => Ok(value.to_owned()),
            None => Err(EvalError::UndefinedVariable(variable_name.to_owned())),
        },
    }
}

/// Substitutes a process/program definition with a path to a file containing
/// the contents of the process' standard output file descriptor.
fn substitute_process(process: &Program, context: &Context) -> EvalResult<String> {
    let mut inner_context = context.try_clone().map_err(EvalError::ContextCloneFailed)?;

    let name: u32 = rand::thread_rng().gen_range(100000..=999999);
    let mut stdout = temp_dir();
    stdout.push(format!("pjsh_{name}_stdout"));
    let stdout_fd = FileDescriptor::File(stdout.clone());
    inner_context.register_temporary_file(stdout.clone());
    inner_context.set_file_descriptor(FD_STDOUT, stdout_fd);

    let stdout_path_string = path_to_string(&stdout);

    execute_subshell(process, inner_context)?;

    Ok(stdout_path_string)
}
