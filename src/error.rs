use std::{
    error::Error as StdError,
    fmt::{Display, Formatter},
    ops::Range,
};

#[derive(Debug)]
pub struct OutOfBoundsError {
    actual_idx: usize,
    bounds: Range<usize>,
}

impl OutOfBoundsError {
    /// Creates new error.
    pub fn new(actual_idx: usize, bounds: Range<usize>) -> Self {
        Self { actual_idx, bounds }
    }
}

impl Display for OutOfBoundsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "index '{}' out of bounds {:?}",
            self.actual_idx, self.bounds,
        )
    }
}

impl StdError for OutOfBoundsError {}

#[derive(Debug)]
pub struct ResizeError {
    details: String,
}

impl ResizeError {
    /// Creates new error with details.
    pub fn new<C>(details: C) -> Self
    where
        C: Into<String>,
    {
        Self {
            details: details.into(),
        }
    }
}

impl Display for ResizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the size of the bitmap cannot be increased: {}",
            self.details
        )
    }
}

impl StdError for ResizeError {}
