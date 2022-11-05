use crate::{List, Word};

/// An iterable value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Iterable {
    /// Iterate over a pre-defined set of items.
    Items(ItemIterable),
    /// Iterate over a range of numeric values.
    Range(NumericRange),
}

impl Iterator for Iterable {
    type Item = Word;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iterable::Items(items) => items.next(),
            Iterable::Range(numeric_range) => numeric_range.next(),
        }
    }
}

impl From<List> for Iterable {
    fn from(list: List) -> Self {
        Iterable::Items(ItemIterable::from(list.items))
    }
}

/// Iterable over a fixed set of items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemIterable {
    /// Items to iterate over.
    items: Vec<Word>,
    /// Current item to return.
    index: usize,
}

impl From<Vec<Word>> for ItemIterable {
    fn from(items: Vec<Word>) -> Self {
        Self { items, index: 0 }
    }
}

impl Iterator for ItemIterable {
    type Item = Word;

    fn next(&mut self) -> Option<Self::Item> {
        let word = self.items.get(self.index).cloned();

        // Advance the iterator until it reaches its end.
        if word.is_some() {
            self.index += 1;
        }

        word
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
