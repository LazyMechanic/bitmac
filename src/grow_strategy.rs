use crate::ResizeError;

/// Determines strategy of bitmap container growth.
pub trait GrowStrategy {
    /// Will be called when the bitmap needs to extend its container.
    /// New length always >= minimal required length of container.
    ///
    /// If returns `Err(_)` then container is not resized.
    fn try_grow(
        &mut self,
        min_req_len: MinimumRequiredLength,
        old_len: usize,
        bit_idx: usize,
    ) -> Result<FinalLength, ResizeError>;

    /// Checks if the container should grow if the changing bit is exceeding container's length
    /// and its new state is `0` (`false`)
    ///
    /// For performance reasons, all bits outside of the container access are
    /// considered to be `0`. The default behavior is to return `false`.
    fn is_force_grow(&self) -> bool {
        false
    }
}

/// Increases the size of the container to the minimum required size.
///
/// Example:
/// ```
/// use bitmac::grow_strategy::{GrowStrategy, MinimumRequiredStrategy, MinimumRequiredLength};
/// let mut s = MinimumRequiredStrategy;
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 0, 0).unwrap().value(), 1);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 0, 10).unwrap().value(), 2);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 0, 23).unwrap().value(), 3);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 3, 24).unwrap().value(), 4);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(5), 3, 35).unwrap().value(), 5);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(6), 3, 47).unwrap().value(), 6);
/// assert!(!s.is_force_grow());
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MinimumRequiredStrategy;

impl GrowStrategy for MinimumRequiredStrategy {
    fn try_grow(
        &mut self,
        min_req_len: MinimumRequiredLength,
        _old_len: usize,
        _bit_idx: usize,
    ) -> Result<FinalLength, ResizeError> {
        Ok(min_req_len.finalize())
    }
}

/// Increases the size of the container by a fixed increment.
///
/// Example:
/// ```
/// use bitmac::grow_strategy::{GrowStrategy, FixedStrategy, MinimumRequiredLength};
/// let mut s = FixedStrategy(3);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 0, 0).unwrap().value(), 3);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 0, 10).unwrap().value(), 3);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 0, 23).unwrap().value(), 3);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 3, 24).unwrap().value(), 6);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(5), 3, 35).unwrap().value(), 6);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(6), 3, 47).unwrap().value(), 6);
/// assert!(!s.is_force_grow());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FixedStrategy(pub usize);

impl GrowStrategy for FixedStrategy {
    fn try_grow(
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
/// ```
/// use bitmac::grow_strategy::{GrowStrategy, MinimumRequiredStrategy, LimitStrategy, MinimumRequiredLength};
/// let mut s = LimitStrategy{
///     strategy: MinimumRequiredStrategy,
///     limit: 5,
/// };
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 0, 0).unwrap().value(), 1);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 0, 10).unwrap().value(), 2);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 0, 23).unwrap().value(), 3);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 3, 24).unwrap().value(), 4);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(5), 3, 35).unwrap().value(), 5);
/// assert!(s.try_grow(MinimumRequiredLength::new_unchecked(6), 3, 47).is_err());
/// assert!(!s.is_force_grow());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LimitStrategy<S> {
    pub strategy: S,
    pub limit: usize,
}

impl<S> GrowStrategy for LimitStrategy<S>
where
    S: GrowStrategy,
{
    fn try_grow(
        &mut self,
        min_req_len: MinimumRequiredLength,
        old_len: usize,
        bit_idx: usize,
    ) -> Result<FinalLength, ResizeError> {
        let final_length = self.strategy.try_grow(min_req_len, old_len, bit_idx)?;
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

/// Increases the size of the container despite new bit state is `0` (`false`).
/// In other words `is_force_grow()` always returns `true`.
///
/// Example:
/// ```
/// use bitmac::grow_strategy::{GrowStrategy, MinimumRequiredStrategy, ForceGrowStrategy, MinimumRequiredLength};
/// let mut s = ForceGrowStrategy(MinimumRequiredStrategy);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 0, 0).unwrap().value(), 1);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 0, 10).unwrap().value(), 2);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 0, 23).unwrap().value(), 3);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 3, 24).unwrap().value(), 4);
/// assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(5), 3, 35).unwrap().value(), 5);
/// assert!(s.is_force_grow());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForceGrowStrategy<S>(pub S);

impl<S> GrowStrategy for ForceGrowStrategy<S>
where
    S: GrowStrategy,
{
    fn try_grow(
        &mut self,
        min_req_len: MinimumRequiredLength,
        old_len: usize,
        bit_idx: usize,
    ) -> Result<FinalLength, ResizeError> {
        self.0.try_grow(min_req_len, old_len, bit_idx)
    }

    fn is_force_grow(&self) -> bool {
        true
    }
}

/// Minimum required length of bitmap container for storing Nth bit.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct MinimumRequiredLength(pub(crate) usize);

impl MinimumRequiredLength {
    /// Increases length by `v` and finalizes it.
    #[inline]
    pub fn advance_by(self, v: usize) -> FinalLength {
        FinalLength(self.0 + v)
    }

    /// Finalizes length and convert it to `FinalLength`.
    #[inline]
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
        
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 1, 0).unwrap().value(), 1);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 2, 0).unwrap().value(), 1);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 3, 0).unwrap().value(), 1);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 4, 0).unwrap().value(), 1);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 5, 0).unwrap().value(), 1);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 1, 0).unwrap().value(), 2);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 2, 0).unwrap().value(), 2);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 3, 0).unwrap().value(), 2);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 4, 0).unwrap().value(), 2);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 5, 0).unwrap().value(), 2);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 5, 0).unwrap().value(), 3);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(21), 5, 0).unwrap().value(), 21);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(25), 5, 0).unwrap().value(), 25);
    }

    #[test]
    #[rustfmt::skip]
    fn test_fixed() {
        let mut s = FixedStrategy(3);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 5, 0).unwrap().value(), 3);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 5, 0).unwrap().value(), 3);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 5, 0).unwrap().value(), 3);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 1, 0).unwrap().value(), 6);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 2, 0).unwrap().value(), 6);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 3, 0).unwrap().value(), 6);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 4, 0).unwrap().value(), 6);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 5, 0).unwrap().value(), 6);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(21), 5, 0).unwrap().value(), 21);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(25), 5, 0).unwrap().value(), 27);
    }

    #[test]
    #[rustfmt::skip]
    fn test_limit() {
        let mut s = LimitStrategy{ strategy: MinimumRequiredStrategy, limit: 3 };

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 1, 0).unwrap().value(), 1);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 2, 0).unwrap().value(), 1);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 3, 0).unwrap().value(), 1);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 4, 0).unwrap().value(), 1);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(1), 5, 0).unwrap().value(), 1);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 1, 0).unwrap().value(), 2);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 2, 0).unwrap().value(), 2);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 3, 0).unwrap().value(), 2);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 4, 0).unwrap().value(), 2);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(2), 5, 0).unwrap().value(), 2);

        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 1, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 2, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 3, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 4, 0).unwrap().value(), 3);
        assert_eq!(s.try_grow(MinimumRequiredLength::new_unchecked(3), 5, 0).unwrap().value(), 3);

        assert!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 1, 0).is_err());
        assert!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 2, 0).is_err());
        assert!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 3, 0).is_err());
        assert!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 4, 0).is_err());
        assert!(s.try_grow(MinimumRequiredLength::new_unchecked(4), 5, 0).is_err());

        assert!(s.try_grow(MinimumRequiredLength::new_unchecked(21), 5, 0).is_err());
        assert!(s.try_grow(MinimumRequiredLength::new_unchecked(25), 5, 0).is_err());
    }
}
