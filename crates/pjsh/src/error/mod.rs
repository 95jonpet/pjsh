use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use pjsh_parse::ParseError;

use crate::shell::ShellError;

/// Generalized error handler.
pub(crate) trait ErrorHandler {
    /// Displays an error.
    fn display_error(&self, error: ShellError);
}

/// A simple error handler, displaying errors on a single line.
pub(crate) struct SimpleErrorHandler;
impl ErrorHandler for SimpleErrorHandler {
    fn display_error(&self, error: ShellError) {
        match error {
            ShellError::Error(error) => eprintln!("pjsh: {error}"),
            ShellError::ParseError(error, _) => eprintln!("pjsh: {error}"),
            ShellError::EvalError(error) => eprintln!("pjsh: {error}"),
            ShellError::IoError(error) => eprintln!("pjsh: {error}"),
        }
    }
}

/// An guiding error handler, displaying errors and help.
pub(crate) struct GuidingErrorHandler;
impl ErrorHandler for GuidingErrorHandler {
    fn display_error(&self, error: ShellError) {
        match error {
            ShellError::Error(error) => eprintln!("pjsh: {error}"),
            ShellError::ParseError(error, line) => {
                print_parse_error_details(&line, &error);
            }
            ShellError::EvalError(error) => eprintln!("pjsh: {error}"),
            ShellError::IoError(error) => eprintln!("pjsh: {error}"),
        }
    }
}

/// Prints details related to a parse error.
fn print_parse_error_details(line: &str, error: &ParseError) {
    let Some(span) = error.span() else {
        eprintln!("pjsh: {error}");
        return;
    };

    let snippet = Snippet {
        title: Some(Annotation {
            label: Some("parse error"),
            id: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
        slices: vec![Slice {
            source: line,
            line_start: 1,
            origin: None,
            fold: true,
            annotations: vec![SourceAnnotation {
                label: error.help(),
                annotation_type: AnnotationType::Error,
                range: (span.start, span.end),
            }],
        }],
        opt: FormatOptions {
            color: true,
            ..Default::default()
        },
    };

    println!("{}", DisplayList::from(snippet));
}
