use pjsh_exec::Executor;

use crate::action;

/// Creates a new executor with registered actions.
pub(crate) fn create_executor() -> Executor {
    let mut executor = Executor::default();
    executor.register_action(Box::new(action::Interpolate {}));
    executor.register_action(Box::new(action::Sleep {}));
    executor.register_action(Box::new(action::Source {}));
    executor
}
