use crate::Word;

/// An iterable value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Iterable {
    /// Iterate over a range of numeric values.
    NumericRange(NumericRange),
}

impl Iterator for Iterable {
    type Item = Word;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iterable::NumericRange(numeric_range) => numeric_range.next(),
        }
    }
}

/// A numeric range iterates between two values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericRange {
    /// The next value.
    next: isize,
    /// The last, and final, value in the range.
    last: isize,
    /// The iteration direction.
    direction: NumericRangeDirection,
}

impl NumericRange {
    pub fn new(start: isize, end: isize) -> Self {
        let direction = if start > end {
            NumericRangeDirection::Decrement
        } else {
            NumericRangeDirection::Increment
        };

        Self {
            next: start,
            last: end,
            direction,
        }
    }
}

impl Iterator for NumericRange {
    type Item = Word;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next != self.last {
            let current = self.next;
            self.next = self.direction.next(current);
            Some(Word::Literal(current.to_string()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NumericRangeDirection {
    /// Iterate such that next > current.
    Increment,
    /// Iterate such that next < current.
    Decrement,
}

impl NumericRangeDirection {
    fn next(&self, current: isize) -> isize {
        match self {
            NumericRangeDirection::Increment => current + 1,
            NumericRangeDirection::Decrement => current - 1,
        }
    }
}
