use pjsh_core::{command::Args, Condition};

/// A condition that is met when a wrapped condition is not met.
#[derive(Clone)]
pub struct Invert(pub Box<dyn Condition>);
impl Condition for Invert {
    fn evaluate(&self, args: Args) -> bool {
        !self.0.evaluate(args)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;

    use super::*;

    #[derive(Clone)]
    struct True;
    impl Condition for True {
        fn evaluate(&self, _: Args) -> bool {
            true
        }
    }

    fn args() -> Args {
        Args {
            context: Arc::new(Mutex::new(pjsh_core::Context::default())),
            io: pjsh_core::command::Io {
                stdin: Box::new(std::io::empty()),
                stdout: Box::new(std::io::sink()),
                stderr: Box::new(std::io::sink()),
            },
        }
    }

    #[test]
    fn equal() {
        assert!(!Invert(Box::new(True {})).evaluate(args()));
        assert!(Invert(Box::new(Invert(Box::new(True {})))).evaluate(args()));
    }
}
