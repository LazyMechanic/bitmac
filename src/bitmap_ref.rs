use std::fmt::{Debug, Formatter};

use crate::{bit_order::BitOrder, get_impl, set_impl, BITS_IN_BYTE};

/// Borrowed bitmap. Helpful if you have already allocated bytes
/// and you want to just look at them as bitmap, without modification.
///
/// Usage example:
/// ```
/// # use bitmac::{BitmapRef, BitOrder};
/// let bitmap = BitmapRef::new(&[0b0000_1000, 0b0000_0001], BitOrder::LSB);
///
/// assert_eq!(bitmap.get(3), true);
/// assert_eq!(bitmap.get(8), true);
///
/// assert_eq!(bitmap.get(1), false);
/// assert_eq!(bitmap.get(7), false);
/// assert_eq!(bitmap.get(300), false);
///
/// assert_eq!(bitmap.as_bytes().len(), 2);
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct BitmapRef<'a> {
    data: &'a [u8],
    bit_order: BitOrder,
}

impl<'a> BitmapRef<'a> {
    pub fn new(data: &'a [u8], bit_order: BitOrder) -> Self {
        Self { data, bit_order }
    }

    /// Get bit state.
    pub fn get(&self, idx: usize) -> bool {
        get_impl(&self.data, &self.bit_order, idx)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Debug for BitmapRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dl = f.debug_list();
        for el in self.data.iter() {
            dl.entry(&format_args!("{:08b}", el));
        }
        dl.finish()
    }
}

/// Borrowed mutable bitmap. Helpful if you have already allocated bytes
/// and you want to just look at them as bitmap, *with* modification.
/// Can't grow up.
///
/// Usage example:
/// ```
/// # use bitmac::{BitmapRefMut, BitOrder};
/// let mut data = [0b0000_1000, 0b0000_0001];
/// let mut bitmap = BitmapRefMut::new(&mut data, BitOrder::LSB);
///
/// assert_eq!(bitmap.set(0, true), true);
/// assert_eq!(bitmap.set(2, true), true);
/// assert_eq!(bitmap.set(300, true), false);
///
/// assert_eq!(bitmap.get(3), true);
/// assert_eq!(bitmap.get(8), true);
///
/// assert_eq!(bitmap.get(1), false);
/// assert_eq!(bitmap.get(7), false);
/// assert_eq!(bitmap.get(300), false);
///
/// assert_eq!(bitmap.as_bytes().len(), 2);
/// ```
#[derive(Eq, PartialEq)]
pub struct BitmapRefMut<'a> {
    data: &'a mut [u8],
    bit_order: BitOrder,
}

impl<'a> BitmapRefMut<'a> {
    pub fn new(data: &'a mut [u8], bit_order: BitOrder) -> Self {
        Self { data, bit_order }
    }

    /// Set bit to specified state.
    /// If index out of bounds then returns `false`, otherwise returns `true`.
    pub fn set(&mut self, idx: usize, v: bool) -> bool {
        let max_idx = self.data.len() * BITS_IN_BYTE;
        if (0..max_idx).contains(&idx) {
            set_impl(self.data, &self.bit_order, idx, v);
            true
        } else {
            false
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

impl Debug for BitmapRefMut<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dl = f.debug_list();
        for el in self.data.iter() {
            dl.entry(&format_args!("{:08b}", el));
        }
        dl.finish()
    }
}
