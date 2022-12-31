use std::path::PathBuf;

use pjsh_ast::Function;
use pjsh_core::{command::Command, find_in_path, Context};

/// A resolved command.
pub(crate) enum ResolvedCommand {
    /// A built-in shell command.
    Builtin(Box<dyn Command>),

    /// A function.
    Function(Function),

    /// A program.
    Program(PathBuf),

    /// Unknown command. Resolution has failed.
    Unknown,
}

/// Resolves a command.
pub(crate) fn resolve_command(name: &str, context: &Context) -> ResolvedCommand {
    if let Some(builtin) = context.get_builtin(&name).map(|cmd| cmd.clone_box()) {
        return ResolvedCommand::Builtin(builtin);
    }

    if let Some(function) = context.get_function(&name).cloned() {
        return ResolvedCommand::Function(function);
    }

    if let Some(program) = find_in_path(&name, context) {
        return ResolvedCommand::Program(program);
    }

    return ResolvedCommand::Unknown;
}
