mod builtins;
mod error;
mod executor;
mod exit;
mod expand;
mod io;
mod word;

#[cfg(test)]
mod tests;

pub use executor::Executor;
pub use expand::expand;
pub use io::{FileDescriptors, Input};
pub use word::interpolate_word;
