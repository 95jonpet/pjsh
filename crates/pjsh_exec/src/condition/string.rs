use pjsh_core::{command::Args, Condition};

/// A condition that is met when two strings are equal to each other.
#[derive(Clone)]
pub struct Equal(pub String, pub String);
impl Condition for Equal {
    fn evaluate(&self, _: Args) -> bool {
        self.0 == self.1
    }
}

/// A condition that is met when two strings are not equal to each other.
#[derive(Clone)]
pub struct NotEqual(pub String, pub String);
impl Condition for NotEqual {
    fn evaluate(&self, _: Args) -> bool {
        self.0 != self.1
    }
}

/// A condition that is met when a string is empty.
#[derive(Clone)]
pub struct Empty(pub String);
impl Condition for Empty {
    fn evaluate(&self, _: Args) -> bool {
        self.0.is_empty()
    }
}

/// A condition that is met when a string is not empty.
#[derive(Clone)]
pub struct NotEmpty(pub String);
impl Condition for NotEmpty {
    fn evaluate(&self, _: Args) -> bool {
        !self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> Args {
        Args {
            context: pjsh_core::Context::default(),
            io: pjsh_core::command::Io {
                stdin: Box::new(std::io::empty()),
                stdout: Box::new(std::io::sink()),
                stderr: Box::new(std::io::sink()),
            },
        }
    }

    #[test]
    fn equal() {
        assert!(Equal("a".into(), "a".into()).evaluate(args()));
        assert!(!Equal("a".into(), "b".into()).evaluate(args()));
    }

    #[test]
    fn not_equal() {
        assert!(NotEqual("a".into(), "b".into()).evaluate(args()));
        assert!(!NotEqual("a".into(), "a".into()).evaluate(args()));
    }

    #[test]
    fn empty() {
        assert!(Empty("".into()).evaluate(args()));
        assert!(!Empty("not-empty".into()).evaluate(args()));
    }

    #[test]
    fn not_empty() {
        assert!(NotEmpty("not-empty".into()).evaluate(args()));
        assert!(!NotEmpty("".into()).evaluate(args()));
    }
}
