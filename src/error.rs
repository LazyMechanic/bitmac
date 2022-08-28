use std::ops::Range;

#[derive(Debug, thiserror::Error)]
#[error("index '{actual_idx}' out of bounds {bounds:?}")]
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

#[derive(Debug, thiserror::Error)]
#[error("the size of the bitmap cannot be increased: {details}")]
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

#[derive(Debug, thiserror::Error)]
#[error("creation of a container with the specified number of slots failed: {details}")]
pub struct WithSlotsError {
    details: String,
}

impl WithSlotsError {
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

#[derive(Debug, thiserror::Error)]
#[error("container size is small: {details}")]
pub struct SmallContainerSizeError {
    details: String,
}

impl SmallContainerSizeError {
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

#[derive(Debug, thiserror::Error)]
pub enum IntersectionError {
    #[error(transparent)]
    SmallContainerSizeError(#[from] SmallContainerSizeError),
    #[error(transparent)]
    WithSlotsError(#[from] WithSlotsError),
}

#[derive(Debug, thiserror::Error)]
pub enum UnionError {
    #[error(transparent)]
    SmallContainerSizeError(#[from] SmallContainerSizeError),
    #[error(transparent)]
    WithSlotsError(#[from] WithSlotsError),
}
