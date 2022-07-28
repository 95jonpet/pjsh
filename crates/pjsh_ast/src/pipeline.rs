use crate::{Command, Word};

/// A pipeline allows multiple programs to be connected using "pipes", sending
/// one program's output as input for another program.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pipeline {
    /// Whether or not to execute the pipeline asynchronously.
    ///
    /// Asynchronous pipelines are not waited for when evaluated.
    pub is_async: bool,

    /// Individual pipeline segments arranged such that the `n`-th segment writes
    /// its output to the input of the `(n+1)`-th segment. The first segment reads
    /// its input from the standard input file descriptor, and the last segment
    /// writes its output to the standard output file descriptor.
    pub segments: Vec<PipelineSegment>,
}

/// A pipeline segment is a single pipable command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineSegment {
    /// A pipable command.
    Command(Command),

    /// A pipable condition.
    /// TODO: How does this type of piping work?
    /// TODO: Consider extracting discrete conditions.
    Condition(Vec<Word>),
}
