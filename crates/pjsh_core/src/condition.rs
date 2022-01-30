use crate::command::Args;

/// A condition is something that can be evaluated as a [`bool`] by the shell.
pub trait Condition: ConditionClone + Send + Sync {
    /// Evaluates the condition.
    fn evaluate(&self, args: Args) -> bool;
}

/// Helper trait for making it easier to clone `Box<Condition>`.
pub trait ConditionClone {
    fn clone_box(&self) -> Box<dyn Condition>;
}

impl<T> ConditionClone for T
where
    T: 'static + Condition + Clone,
{
    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Condition> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
