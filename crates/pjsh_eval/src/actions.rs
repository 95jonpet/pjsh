use pjsh_core::{
    command::{Action, CommandType},
    find_in_path, Context,
};
use pjsh_parse::parse_interpolation;

use crate::{
    interpolate_word,
    resolve::{resolve_command, ResolvedCommand},
    EvalResult,
};

/// Handles an action.
pub(crate) fn handle_action(action: &Action, context: &mut Context) -> EvalResult<()> {
    match action {
        Action::ExitScope(_code) => todo!(),
        Action::Interpolate(word, callback) => {
            let result = parse_interpolation(word)
                .map_err(|error| format!("{error}"))
                .and_then(|word| {
                    interpolate_word(&word, context).map_err(|error| format!("{error}"))
                });
            callback(context.io(), result);
            Ok(())
        }
        Action::ResolveCommandType(name, callback) => {
            let command_type = if let Some(alias) = context.aliases.get(name) {
                CommandType::Alias(alias.clone())
            } else {
                match resolve_command(name, context) {
                    ResolvedCommand::Builtin(_) => CommandType::Builtin,
                    ResolvedCommand::Function(_) => CommandType::Function,
                    ResolvedCommand::Program(path) => CommandType::Program(path),
                    ResolvedCommand::Unknown => CommandType::Unknown,
                }
            };

            callback(context.io(), name.clone(), command_type);
            Ok(())
        }
        Action::ResolveCommandPath(name, callback) => {
            let path = find_in_path(name, context);
            callback(name.clone(), context.io(), path.as_ref());
            Ok(())
        }
    }
}
