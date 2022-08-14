use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
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

impl Error for ResizeError {}
