use crate::ResizeError;

/// Determines strategy of bitmap container resizing.
pub trait ResizingStrategy {
    /// Will be called when the bitmap needs to extend its container.
    /// New length always >= minimal required length of container.
    ///
    /// If returns `Err(_)` then container is not resized.
    fn try_resize(
        &mut self,
        min_req_len: MinimumRequiredLength,
        old_len: usize,
        bit_idx: usize,
    ) -> Result<FinalLength, ResizeError>;

    /// Will be called when a bitmap needs to resize its container to put a value,
    /// but new bit value is `0` (`false`), which means container resizing is optional.
    ///
    /// For performance reasons, all bits outside of the container access are
    /// considered to be `0`. The default behavior is to return `Ok(None)`.
    fn try_resize_opt(
        &mut self,
        _min_req_len: MinimumRequiredLength,
        _old_len: usize,
        _bit_idx: usize,
    ) -> Result<Option<FinalLength>, ResizeError> {
        Ok(None)
    }
}

/// Increases the size of the container to the minimum required size.
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
        _old_len: usize,
        _bit_idx: usize,
    ) -> Result<FinalLength, ResizeError> {
        if min_req_len.value() % self.0 == 0 {
            Ok(min_req_len.finalize())
        } else {
            let rest = (min_req_len.value() / self.0 + 1) * self.0 - min_req_len.value();
            Ok(min_req_len.advance_by(rest))
        }
    }
}

/// Increases the size of the container until the limit is reached.
///
/// Example:
/// ```no_run
/// use bitmac::resizing_strategy::{ResizingStrategy, MinimumRequiredStrategy, LimitStrategy, MinimumRequiredLength};
/// let mut s = LimitStrategy{
///     strategy: MinimumRequiredStrategy,
///     limit: 5,
/// };
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 0, 0).unwrap().value(), 1);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 0, 10).unwrap().value(), 2);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 0, 23).unwrap().value(), 3);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 3, 24).unwrap().value(), 4);
/// assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(5), 3, 35).unwrap().value(), 5);
/// assert!(s.try_resize(MinimumRequiredLength::new_unchecked(6), 3, 47).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LimitStrategy<S> {
    pub strategy: S,
    pub limit: usize,
}

impl<S> ResizingStrategy for LimitStrategy<S>
where
    S: ResizingStrategy,
{
    fn try_resize(
        &mut self,
        min_req_len: MinimumRequiredLength,
        old_len: usize,
        bit_idx: usize,
    ) -> Result<FinalLength, ResizeError> {
        let final_length = self.strategy.try_resize(min_req_len, old_len, bit_idx)?;
        if final_length.value() <= self.limit {
            Ok(final_length)
        } else {
            Err(ResizeError::new(format!(
                "the new size {} is over the limit {}",
                final_length.value(),
                self.limit
            )))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_minimal() {
        let mut s = MinimumRequiredStrategy::default();
        
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 1, 0).unwrap().value(), 1);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 2, 0).unwrap().value(), 1);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 3, 0).unwrap().value(), 1);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 4, 0).unwrap().value(), 1);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 5, 0).unwrap().value(), 1);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 1, 0).unwrap().value(), 2);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 2, 0).unwrap().value(), 2);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 3, 0).unwrap().value(), 2);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 4, 0).unwrap().value(), 2);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 5, 0).unwrap().value(), 2);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 5, 0).unwrap().value(), 3);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(21), 5, 0).unwrap().value(), 21);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(25), 5, 0).unwrap().value(), 25);
    }

    #[test]
    #[rustfmt::skip]
    fn test_fixed() {
        let mut s = FixedStrategy(3);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 5, 0).unwrap().value(), 3);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 5, 0).unwrap().value(), 3);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 5, 0).unwrap().value(), 3);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 1, 0).unwrap().value(), 6);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 2, 0).unwrap().value(), 6);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 3, 0).unwrap().value(), 6);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 4, 0).unwrap().value(), 6);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 5, 0).unwrap().value(), 6);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(21), 5, 0).unwrap().value(), 21);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(25), 5, 0).unwrap().value(), 27);
    }

    #[test]
    #[rustfmt::skip]
    fn test_limit() {
        let mut s = LimitStrategy{ strategy: MinimumRequiredStrategy, limit: 3 };

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 1, 0).unwrap().value(), 1);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 2, 0).unwrap().value(), 1);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 3, 0).unwrap().value(), 1);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 4, 0).unwrap().value(), 1);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(1), 5, 0).unwrap().value(), 1);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 1, 0).unwrap().value(), 2);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 2, 0).unwrap().value(), 2);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 3, 0).unwrap().value(), 2);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 4, 0).unwrap().value(), 2);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(2), 5, 0).unwrap().value(), 2);

        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_resize(MinimumRequiredLength::new_unchecked(3), 5, 0).unwrap().value(), 3);

        assert!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 1, 0).is_err());
        assert!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 2, 0).is_err());
        assert!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 3, 0).is_err());
        assert!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 4, 0).is_err());
        assert!(s.try_resize(MinimumRequiredLength::new_unchecked(4), 5, 0).is_err());

        assert!(s.try_resize(MinimumRequiredLength::new_unchecked(21), 5, 0).is_err());
        assert!(s.try_resize(MinimumRequiredLength::new_unchecked(25), 5, 0).is_err());
    }
}
