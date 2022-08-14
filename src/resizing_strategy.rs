use crate::ResizeError;

/// Determines strategy of bitmap container growing.
pub trait ResizingStrategy {
    /// Will be called when the bitmap needs to extend its container.
    /// New length always >= minimal required length of container.
    /// If returns `Err(_)` then container is not extended.
    fn try_resize(
        &mut self,
        min_req_len: MinimumRequiredLength,
        old_len: usize,
        bit_idx: usize,
    ) -> Result<FinalLength, ResizeError>;
}

/// Increases the size of the container in a minimum of required increment.
///
/// Example:
/// ```
/// use bitmac::resizing_strategy::{ResizingStrategy, MinimumRequiredStrategy, MinimumRequiredLength};
/// let mut s = MinimumRequiredStrategy;
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 0, 0).unwrap().value(), 1);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 0, 10).unwrap().value(), 2);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 0, 23).unwrap().value(), 3);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 3, 24).unwrap().value(), 4);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(5), 3, 35).unwrap().value(), 5);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(6), 3, 47).unwrap().value(), 6);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MinimumRequiredStrategy;

impl ResizingStrategy for MinimumRequiredStrategy {
    fn try_resize(
        &mut self,
        min_req_len: MinimumRequiredLength,
        _old_len: usize,
        _bit_idx: usize,
    ) -> Result<FinalLength, ResizeError> {
        Ok(min_req_len.finalize())
    }
}

/// Increases the size of the container in a fixed increment.
///
/// Example:
/// ```no_run
/// use bitmac::resizing_strategy::{ResizingStrategy, FixedStrategy, MinimumRequiredLength};
/// let mut s = FixedStrategy(3);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 0, 0).unwrap().value(), 3);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 0, 10).unwrap().value(), 3);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 0, 23).unwrap().value(), 3);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 3, 24).unwrap().value(), 6);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(5), 3, 35).unwrap().value(), 6);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(6), 3, 47).unwrap().value(), 6);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FixedStrategy(pub usize);

impl ResizingStrategy for FixedStrategy {
    fn try_resize(
        &mut self,
        min_req_len: MinimumRequiredLength,
        old_len: usize,
        _bit_idx: usize,
    ) -> Result<FinalLength, ResizeError> {
        let rest = usize::max(min_req_len.value() / self.0, 1) * self.0 - old_len;
        Ok(min_req_len.advance_by(rest))
    }
}

/// Strategy that doesn't support resizing.
///
/// Example:
/// ```
/// use bitmac::resizing_strategy::{ResizingStrategy, StaticStrategy, MinimumRequiredLength};
/// let mut s = StaticStrategy;
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 1, 0).unwrap().value(), 1);
/// assert!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 1, 10).is_err());
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StaticStrategy;

impl ResizingStrategy for StaticStrategy {
    fn try_resize(
        &mut self,
        min_req_len: MinimumRequiredLength,
        old_len: usize,
        _bit_idx: usize,
    ) -> Result<FinalLength, ResizeError> {
        if min_req_len.value() != old_len {
            Err(ResizeError::new(
                "static resizing strategy doesn't support resizing",
            ))
        } else {
            Ok(min_req_len.finalize())
        }
    }
}

/// Minimum required length of bitmap container for storing Nth bit.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct MinimumRequiredLength(pub(crate) usize);

impl MinimumRequiredLength {
    /// Increase length by `v` and finalize length.
    #[inline]
    pub fn advance_by(self, v: usize) -> FinalLength {
        FinalLength(self.0 + v)
    }

    /// Finalize length and convert it to `FinalLength`
    pub fn finalize(self) -> FinalLength {
        FinalLength(self.0)
    }

    /// Returns inner value.
    #[inline]
    pub fn value(&self) -> usize {
        self.0
    }

    /// Creates `MinimumRequiredLength`. For testing and document purposes only.
    #[doc(hidden)]
    pub fn new_unchecked(v: usize) -> Self {
        Self(v)
    }
}

/// New bitmap container length.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct FinalLength(pub(crate) usize);

impl FinalLength {
    /// Returns inner value.
    #[inline]
    pub fn value(&self) -> usize {
        self.0
    }
}
