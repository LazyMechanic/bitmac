use std::fmt::{Debug, Formatter};

use crate::{bit_order::BitOrder, extender::Extender, BITS_IN_BYTE};

/// Owned bitmap.
///
/// Usage example:
/// ```
/// # use bitmac::{Bitmap, BitOrder, MinimumRequiredExtender};
/// let mut bitmap = Bitmap::with_capacity(1, MinimumRequiredExtender, BitOrder::LSB);
///
/// bitmap.set(0, true);
/// bitmap.set(7, true);
/// bitmap.set(15, true);
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
pub struct Bitmap<E> {
    data: Vec<u8>,
    extender: E,
    bit_order: BitOrder,
}

impl<E> Bitmap<E>
where
    E: Extender,
{
    pub fn new(extender: E, bit_order: BitOrder) -> Self {
        Self {
            data: vec![],
            extender,
            bit_order,
        }
    }

    pub fn with_capacity(bytes_cap: usize, extender: E, bit_order: BitOrder) -> Self {
        Self {
            data: Vec::with_capacity(bytes_cap),
            extender,
            bit_order,
        }
    }

    /// Set bit to specified state. If container smaller that needs then extends it.
    pub fn set(&mut self, idx: usize, v: bool) {
        fn set_impl(data: &mut [u8], bo: &BitOrder, idx: usize, v: bool) {
            let bit_idx = 0b0111 - (idx & 0b0111);
            let byte_idx = idx >> 3;

            let byte = &mut data[byte_idx];
            *byte = bo.set(*byte, bit_idx, v);
        }

        let max_idx = self.data.len() * BITS_IN_BYTE;
        match idx {
            // Index fits in the bitmap
            idx if (0..max_idx).contains(&idx) => {
                set_impl(&mut self.data, &self.bit_order, idx, v);
            }
            // Index does not fit in the bitmap
            idx => {
                let required_len = self.data.len() + (idx - max_idx) / BITS_IN_BYTE + 1;
                let extended_len = self.extender.extend(self.data.len(), idx);
                let new_len = usize::max(required_len, extended_len);
                self.data.resize(new_len, 0u8);

                set_impl(&mut self.data, &self.bit_order, idx, v);
            }
        }
    }

    /// Set bit state.
    pub fn get(&self, idx: usize) -> bool {
        let bit_idx = 0b0111 - (idx & 0b0111);
        let byte_idx = idx >> 3;

        // If idx out of bounds
        if byte_idx >= self.data.len() {
            return false;
        }

        self.bit_order.get(self.data[byte_idx], bit_idx)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl<E> Debug for Bitmap<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dl = f.debug_list();
        for el in self.data.iter() {
            dl.entry(&format_args!("{:08b}", el));
        }
        dl.finish()
    }
}
