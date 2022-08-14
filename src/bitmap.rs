use std::fmt::{Debug, Formatter};

use crate::{
    get_impl, set_impl, BitAccess, Container, FinalLength, MinimumRequiredLength, ResizeError,
    ResizingStrategy, BITS_IN_BYTE,
};

/// Bitmap that owns the container.
///
/// Usage example:
/// ```
/// # use bitmac::{Bitmap, LSB, MinimumRequiredStrategy};
/// let mut bitmap = Bitmap::<Vec<u8>, MinimumRequiredStrategy, LSB>::default();
///
/// bitmap.set(0, true);
/// bitmap.set(7, true);
/// assert_eq!(bitmap.as_bytes().len(), 1);
/// bitmap.set(15, true);
/// assert_eq!(bitmap.as_bytes().len(), 2);
///
/// assert_eq!(bitmap.get(0), true);
/// assert_eq!(bitmap.get(7), true);
/// assert_eq!(bitmap.get(15), true);
///
/// assert_eq!(bitmap.get(1), false);
/// assert_eq!(bitmap.get(8), false);
/// assert_eq!(bitmap.get(300), false);
///
/// assert_eq!(bitmap.as_bytes().len(), 2);
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct Bitmap<C, S, B> {
    data: C,
    resizing_strategy: S,
    bit_access: B,
}

impl<C, S, B> Default for Bitmap<C, S, B>
where
    C: Container + Default,
    S: ResizingStrategy + Default,
    B: BitAccess + Default,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
            resizing_strategy: Default::default(),
            bit_access: Default::default(),
        }
    }
}

impl<C, S, B> Bitmap<C, S, B> {
    /// Creates new bitmap from container with specified resizing strategy.
    pub fn new(data: C, resizing_strategy: S) -> Bitmap<C, S, B>
    where
        C: Container,
        S: ResizingStrategy,
        B: BitAccess + Default,
    {
        Bitmap {
            data,
            resizing_strategy,
            bit_access: B::default(),
        }
    }

    /// Creates new bitmap from bytes.
    pub fn from_bytes(data: C) -> Bitmap<C, S, B>
    where
        C: Container,
        S: ResizingStrategy + Default,
        B: BitAccess + Default,
    {
        Bitmap {
            data,
            resizing_strategy: S::default(),
            bit_access: B::default(),
        }
    }

    /// Creates new empty bitmap with specified resizing strategy.
    pub fn with_resizing_strategy(resizing_strategy: S) -> Bitmap<C, S, B>
    where
        C: Container + Default,
        S: ResizingStrategy,
        B: BitAccess + Default,
    {
        Bitmap {
            data: C::default(),
            resizing_strategy,
            bit_access: B::default(),
        }
    }

    /// Creates new bitmap from parts.
    pub fn from_parts(data: C, resizing_strategy: S, bit_access: B) -> Bitmap<C, S, B>
    where
        C: Container,
        S: ResizingStrategy,
        B: BitAccess,
    {
        Bitmap {
            data,
            resizing_strategy,
            bit_access,
        }
    }
}

impl<C, S, B> Bitmap<C, S, B>
where
    C: Container,
    S: ResizingStrategy,
    B: BitAccess,
{
    /// Set bit to specified state. If container smaller that needs and
    /// resizing strategy allowed then resize it.
    ///
    /// If resizing failed then nothing will happen.
    pub fn set(&mut self, idx: usize, v: bool) {
        let _ = self.try_set(idx, v);
    }

    /// Set bit to specified state. If container smaller that needs and
    /// resizing strategy allowed then resize it.
    ///
    /// If resizing failed then return error.
    pub fn try_set(&mut self, idx: usize, v: bool) -> Result<(), ResizeError> {
        let max_idx = self.data.as_ref().len() * BITS_IN_BYTE;
        if idx < max_idx {
            set_impl(self.data.as_mut(), &self.bit_access, idx, v);
        } else {
            // Try to resize container
            let old_len = self.data.as_ref().len();
            let min_req_len = old_len + (idx - max_idx) / BITS_IN_BYTE + 1;
            let min_req_len = MinimumRequiredLength(min_req_len);
            let FinalLength(new_len) =
                self.resizing_strategy
                    .try_resize(min_req_len, old_len, idx)?;

            // Resize container if new length doesn't match old length
            if new_len != old_len {
                self.data.try_resize(new_len, 0u8)?;
            }
            set_impl(self.data.as_mut(), &self.bit_access, idx, v);
        }

        Ok(())
    }

    /// Get bit state.
    pub fn get(&self, idx: usize) -> bool {
        get_impl(self.data.as_ref(), &self.bit_access, idx)
    }

    /// Represents bitmap as slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_ref()
    }

    /// Converts bitmap to inner container.
    pub fn into_inner(self) -> C {
        self.data
    }
}

impl<C, S, B> Debug for Bitmap<C, S, B>
where
    C: Container,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dl = f.debug_list();
        for el in self.data.as_ref() {
            dl.entry(&format_args!("{:08b}", el));
        }
        dl.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DynBitAccess, FixedStrategy, MinimumRequiredStrategy, StaticStrategy, LSB, MSB};

    #[test]
    #[rustfmt::skip]
    fn test_construct() {
        // Default
        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, LSB> = Bitmap::default();
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, LSB> = Bitmap::default();
        let _: Bitmap<Vec<u8>, StaticStrategy, LSB> = Bitmap::default();
        let _: Bitmap<[u8; 32], StaticStrategy, LSB> = Bitmap::default();

        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, MSB> = Bitmap::default();
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, MSB> = Bitmap::default();
        let _: Bitmap<Vec<u8>, StaticStrategy, MSB> = Bitmap::default();
        let _: Bitmap<[u8; 32], StaticStrategy, MSB> = Bitmap::default();

        // new
        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, LSB> = Bitmap::new(Vec::new(), MinimumRequiredStrategy::default());
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, LSB> = Bitmap::new([0u8; 32], MinimumRequiredStrategy::default());
        let _: Bitmap<Vec<u8>, StaticStrategy, LSB> = Bitmap::new(Vec::new(), StaticStrategy::default());
        let _: Bitmap<[u8; 32], StaticStrategy, LSB> = Bitmap::new([0u8; 32], StaticStrategy::default());
        let _: Bitmap<Vec<u8>, FixedStrategy, LSB> = Bitmap::new(Vec::new(), FixedStrategy(5));
        let _: Bitmap<[u8; 32], FixedStrategy, LSB> = Bitmap::new([0u8; 32], FixedStrategy(5));

        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, MSB> = Bitmap::new(Vec::new(), MinimumRequiredStrategy::default());
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, MSB> = Bitmap::new([0u8; 32], MinimumRequiredStrategy::default());
        let _: Bitmap<Vec<u8>, StaticStrategy, MSB> = Bitmap::new(Vec::new(), StaticStrategy::default());
        let _: Bitmap<[u8; 32], StaticStrategy, MSB> = Bitmap::new([0u8; 32], StaticStrategy::default());
        let _: Bitmap<Vec<u8>, FixedStrategy, MSB> = Bitmap::new(Vec::new(), FixedStrategy(5));
        let _: Bitmap<[u8; 32], FixedStrategy, MSB> = Bitmap::new([0u8; 32], FixedStrategy(5));

        // from_bytes
        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, LSB> = Bitmap::from_bytes(Vec::new());
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, LSB> = Bitmap::from_bytes([0u8; 32]);
        let _: Bitmap<Vec<u8>, StaticStrategy, LSB> = Bitmap::from_bytes(Vec::new());
        let _: Bitmap<[u8; 32], StaticStrategy, LSB> = Bitmap::from_bytes([0u8; 32]);

        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, MSB> = Bitmap::from_bytes(Vec::new());
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, MSB> = Bitmap::from_bytes([0u8; 32]);
        let _: Bitmap<Vec<u8>, StaticStrategy, MSB> = Bitmap::from_bytes(Vec::new());
        let _: Bitmap<[u8; 32], StaticStrategy, MSB> = Bitmap::from_bytes([0u8; 32]);

        // with_resizing_strategy
        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, LSB> = Bitmap::with_resizing_strategy(MinimumRequiredStrategy::default());
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, LSB> = Bitmap::with_resizing_strategy(MinimumRequiredStrategy::default());
        let _: Bitmap<Vec<u8>, StaticStrategy, LSB> = Bitmap::with_resizing_strategy(StaticStrategy::default());
        let _: Bitmap<[u8; 32], StaticStrategy, LSB> = Bitmap::with_resizing_strategy(StaticStrategy::default());
        let _: Bitmap<Vec<u8>, FixedStrategy, LSB> = Bitmap::with_resizing_strategy(FixedStrategy(5));
        let _: Bitmap<[u8; 32], FixedStrategy, LSB> = Bitmap::with_resizing_strategy(FixedStrategy(5));

        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, MSB> = Bitmap::with_resizing_strategy(MinimumRequiredStrategy::default());
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, MSB> = Bitmap::with_resizing_strategy(MinimumRequiredStrategy::default());
        let _: Bitmap<Vec<u8>, StaticStrategy, MSB> = Bitmap::with_resizing_strategy(StaticStrategy::default());
        let _: Bitmap<[u8; 32], StaticStrategy, MSB> = Bitmap::with_resizing_strategy(StaticStrategy::default());
        let _: Bitmap<Vec<u8>, FixedStrategy, MSB> = Bitmap::with_resizing_strategy(FixedStrategy(5));
        let _: Bitmap<[u8; 32], FixedStrategy, MSB> = Bitmap::with_resizing_strategy(FixedStrategy(5));

        // from_parts
        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, LSB> = Bitmap::from_parts(Vec::new(), MinimumRequiredStrategy::default(), LSB::default());
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, LSB> = Bitmap::from_parts([0u8; 32], MinimumRequiredStrategy::default(), LSB::default());
        let _: Bitmap<Vec<u8>, StaticStrategy, LSB> = Bitmap::from_parts(Vec::new(), StaticStrategy::default(), LSB::default());
        let _: Bitmap<[u8; 32], StaticStrategy, LSB> = Bitmap::from_parts([0u8; 32], StaticStrategy::default(), LSB::default());
        let _: Bitmap<Vec<u8>, FixedStrategy, LSB> = Bitmap::from_parts(Vec::new(), FixedStrategy(5), LSB::default());
        let _: Bitmap<[u8; 32], FixedStrategy, LSB> = Bitmap::from_parts([0u8; 32], FixedStrategy(5), LSB::default());

        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, MSB> = Bitmap::from_parts(Vec::new(), MinimumRequiredStrategy::default(), MSB::default());
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, MSB> = Bitmap::from_parts([0u8; 32], MinimumRequiredStrategy::default(), MSB::default());
        let _: Bitmap<Vec<u8>, StaticStrategy, MSB> = Bitmap::from_parts(Vec::new(), StaticStrategy::default(), MSB::default());
        let _: Bitmap<[u8; 32], StaticStrategy, MSB> = Bitmap::from_parts([0u8; 32], StaticStrategy::default(), MSB::default());
        let _: Bitmap<Vec<u8>, FixedStrategy, MSB> = Bitmap::from_parts(Vec::new(), FixedStrategy(5), MSB::default());
        let _: Bitmap<[u8; 32], FixedStrategy, MSB> = Bitmap::from_parts([0u8; 32], FixedStrategy(5), MSB::default());

        let _: Bitmap<Vec<u8>, MinimumRequiredStrategy, DynBitAccess> = Bitmap::from_parts(Vec::new(), MinimumRequiredStrategy::default(), DynBitAccess::LSB);
        let _: Bitmap<[u8; 32], MinimumRequiredStrategy, DynBitAccess> = Bitmap::from_parts([0u8; 32], MinimumRequiredStrategy::default(), DynBitAccess::LSB);
        let _: Bitmap<Vec<u8>, StaticStrategy, DynBitAccess> = Bitmap::from_parts(Vec::new(), StaticStrategy::default(), DynBitAccess::LSB);
        let _: Bitmap<[u8; 32], StaticStrategy, DynBitAccess> = Bitmap::from_parts([0u8; 32], StaticStrategy::default(), DynBitAccess::LSB);
        let _: Bitmap<Vec<u8>, FixedStrategy, DynBitAccess> = Bitmap::from_parts(Vec::new(), FixedStrategy(5), DynBitAccess::LSB);
        let _: Bitmap<[u8; 32], FixedStrategy, DynBitAccess> = Bitmap::from_parts([0u8; 32], FixedStrategy(5), DynBitAccess::LSB);
    }
}
