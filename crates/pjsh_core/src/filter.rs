use std::fmt::Display;

use crate::Value;

/// Filter-related errors.
#[derive(Debug, PartialEq, Eq)]
pub enum FilterError {
    /// The filter cannot be applied using the provided arguments.
    InvalidArgs(String),

    /// The filter cannot be applied to lists.
    InvalidListFilter,

    /// The filter cannot be applied to words.
    InvalidWordFilter,

    /// The filter is missing a required argument.
    MissingArg(&'static str),

    /// The filter does not accept any arguments.
    NoArgsAllowed,

    /// The filter does not return a value.
    NoSuchValue,

    /// The filter has been given too many arguments.
    TooManyArgs,
}

/// Specialized result type for filters.
pub type FilterResult = Result<Value, FilterError>;

/// A filter represents a value transformation.
pub trait Filter: FilterClone {
    /// Returns the filter's name.
    fn name(&self) -> &str;

    /// Returns the result of applying the filter on a list.
    fn filter_list(&self, _list: Vec<String>, _args: &[String]) -> FilterResult {
        Err(FilterError::InvalidListFilter)
    }

    /// Returns the result of applying the filter on a word.
    fn filter_word(&self, _word: String, _args: &[String]) -> FilterResult {
        Err(FilterError::InvalidWordFilter)
    }
}

impl Display for FilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterError::InvalidArgs(msg) => {
                write!(f, "invalid arguments for filter: {msg}")
            }
            FilterError::InvalidListFilter => {
                write!(f, "the filter cannot be applied to lists")
            }
            FilterError::InvalidWordFilter => {
                write!(f, "the filter cannot be applied to words")
            }
            FilterError::MissingArg(arg) => write!(f, "missing required argument '{arg}'"),
            FilterError::NoArgsAllowed => {
                write!(f, "the filter does not accept any arguments")
            }
            FilterError::NoSuchValue => write!(f, "no such value"),
            FilterError::TooManyArgs => write!(f, "too many arguments"),
        }
    }
}

/// Helper trait for making it easier to clone `Box<Command>`.
pub trait FilterClone {
    /// Clones the Filter into a new boxed instance.
    fn clone_box(&self) -> Box<dyn Filter>;
}

impl<T> FilterClone for T
where
    T: 'static + Filter + Clone,
{
    fn clone_box(&self) -> Box<dyn Filter> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Filter> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
