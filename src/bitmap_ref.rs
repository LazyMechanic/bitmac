use std::fmt::{Debug, Formatter};

use crate::{get_impl, set_impl, BitAccess, BITS_IN_BYTE};

/// Borrowed bitmap. Helpful if you have already allocated bytes
/// and you want to just look at them as bitmap, without modification.
///
/// Usage example:
/// ```
/// # use bitmac::{BitmapRef, LSB};
/// let bitmap = BitmapRef::<'_, LSB>::new(&[0b0000_1000, 0b0000_0001]);
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
pub struct BitmapRef<'a, B> {
    data: &'a [u8],
    bit_access: B,
}

impl<'a, B> BitmapRef<'a, B> {
    /// Creates new bitmap from bytes.
    pub fn new(data: &'a [u8]) -> Self
    where
        B: BitAccess + Default,
    {
        Self {
            data,
            bit_access: B::default(),
        }
    }

    /// Create new bitmap from parts.
    pub fn from_parts(data: &'a [u8], bit_access: B) -> Self
    where
        B: BitAccess,
    {
        Self { data, bit_access }
    }
}

impl<'a, B> BitmapRef<'a, B>
where
    B: BitAccess,
{
    /// Get bit state.
    pub fn get(&self, idx: usize) -> bool {
        get_impl(self.data, &self.bit_access, idx)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data
    }
}

impl<B> Debug for BitmapRef<'_, B> {
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
/// Can't resized.
///
/// Usage example:
/// ```
/// # use bitmac::{BitmapRefMut, LSB};
/// let mut data = [0b0000_1000, 0b0000_0001];
/// let mut bitmap = BitmapRefMut::<'_, LSB>::new(&mut data);
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
pub struct BitmapRefMut<'a, B> {
    data: &'a mut [u8],
    bit_access: B,
}

impl<'a, B> BitmapRefMut<'a, B> {
    /// Creates new bitmap from bytes.
    pub fn new(data: &'a mut [u8]) -> Self
    where
        B: BitAccess + Default,
    {
        Self {
            data,
            bit_access: B::default(),
        }
    }

    /// Create new bitmap from parts.
    pub fn from_parts(data: &'a mut [u8], bit_access: B) -> Self
    where
        B: BitAccess,
    {
        Self { data, bit_access }
    }
}

impl<'a, B> BitmapRefMut<'a, B>
where
    B: BitAccess,
{
    /// Set bit to specified state.
    /// If index out of bounds then returns `false`, otherwise returns `true`.
    pub fn set(&mut self, idx: usize, v: bool) -> bool {
        let max_idx = self.data.len() * BITS_IN_BYTE;
        if (0..max_idx).contains(&idx) {
            set_impl(self.data, &self.bit_access, idx, v);
            true
        } else {
            false
        }
    }

    /// Get bit state.
    pub fn get(&self, idx: usize) -> bool {
        get_impl(self.data, &self.bit_access, idx)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data
    }
}

impl<B> Debug for BitmapRefMut<'_, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dl = f.debug_list();
        for el in self.data.iter() {
            dl.entry(&format_args!("{:08b}", el));
        }
        dl.finish()
    }
}
