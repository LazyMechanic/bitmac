use std::fmt::{Debug, Formatter};

use crate::{bit_order::BitOrder, extender::Extender, BITS_IN_BYTE};

/// Owned bitmap. Can grow in size if you set bit out of bounds.
///
/// Usage example:
/// ```
/// # use bitmac::{Bitmap, BitOrder, MinimumRequiredExtender};
/// let mut bitmap = Bitmap::with_capacity(1, MinimumRequiredExtender, BitOrder::LSB);
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
pub struct Bitmap<E> {
    data: Vec<u8>,
    extender: E,
    bit_order: BitOrder,
}

impl<E> Bitmap<E>
where
    E: Extender,
{
    /// Creates new empty bitmap.
    pub fn new(extender: E, bit_order: BitOrder) -> Self {
        Self {
            data: vec![],
            extender,
            bit_order,
        }
    }

    /// Creates new bitmap from preallocated bytes.
    pub fn from_data<V>(data: V, extender: E, bit_order: BitOrder) -> Self
    where
        V: Into<Vec<u8>>,
    {
        Self {
            data: data.into(),
            extender,
            bit_order,
        }
    }

    /// Creates new bitmap with preallocation of bytes.
    pub fn with_capacity(bytes_cap: usize, extender: E, bit_order: BitOrder) -> Self {
        Self {
            data: Vec::with_capacity(bytes_cap),
            extender,
            bit_order,
        }
    }

    /// Set bit to specified state. If container smaller that needs then extends it.
    pub fn set(&mut self, idx: usize, v: bool) {
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

    /// Get bit state.
    pub fn get(&self, idx: usize) -> bool {
        get_impl(&self.data, &self.bit_order, idx)
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

pub(crate) fn set_impl(data: &mut [u8], bo: &BitOrder, idx: usize, v: bool) {
    let bit_idx = idx & 0b0111;
    let byte_idx = idx >> 3;

    let byte = &mut data[byte_idx];
    *byte = bo.set(*byte, bit_idx, v);
}

pub(crate) fn get_impl(data: &[u8], bo: &BitOrder, idx: usize) -> bool {
    let bit_idx = idx & 0b0111;
    let byte_idx = idx >> 3;

    // If idx out of bounds
    if byte_idx >= data.len() {
        return false;
    }

    bo.get(data[byte_idx], bit_idx)
}
