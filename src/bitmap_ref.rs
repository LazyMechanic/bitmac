use std::fmt::{Debug, Formatter};

use crate::{get_impl, set_impl, BitAccess, OutOfBoundsError, BITS_IN_BYTE};

/// Bitmap that borrows bytes. Helpful if you have already allocated bytes
/// and you want to just look at them as bitmap, without modifications.
///
/// Usage example:
/// ```
/// # use bitmac::{BitmapRef, LSB};
/// let bitmap = BitmapRef::<'_, LSB>::from_bytes(&[0b0000_1000, 0b0000_0001]);
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
    pub fn from_bytes(data: &'a [u8]) -> Self
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

/// Bitmap that borrows mutable bytes. Helpful if you have already allocated bytes
/// and you want to just look at them as bitmap and modify it.
/// Cannot increase the number of bytes.
///
/// Usage example:
/// ```
/// # use bitmac::{BitmapRefMut, LSB};
/// let mut data = [0b0000_1000, 0b0000_0001];
/// let mut bitmap = BitmapRefMut::<'_, LSB>::from_bytes(&mut data);
///
/// assert_eq!(bitmap.get(3), true);
/// assert_eq!(bitmap.get(8), true);
///
/// bitmap.set(0, true);
/// bitmap.set(2, true);
/// bitmap.set(300, true);
/// assert_eq!(bitmap.get(0), true);
/// assert_eq!(bitmap.get(2), true);
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
    pub fn from_bytes(data: &'a mut [u8]) -> Self
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
    /// If index out of bounds then nothing will happen.
    pub fn set(&mut self, idx: usize, v: bool) {
        let _ = self.try_set(idx, v);
    }

    /// Set bit to specified state.
    ///
    /// If index out of bounds then returns `Err(_)`, otherwise returns `Ok(())`.
    pub fn try_set(&mut self, idx: usize, v: bool) -> Result<(), OutOfBoundsError> {
        let max_idx = self.data.len() * BITS_IN_BYTE;
        if idx < max_idx {
            set_impl(self.data, &self.bit_access, idx, v);
            Ok(())
        } else {
            Err(OutOfBoundsError::new(idx, 0..self.data.len()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LSB, MSB};

    #[test]
    fn bitmap_ref_lsb() {
        let v = [
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0001,
            0b1000_1000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
        ];
        let bitmap = BitmapRef::<'_, LSB>::from_bytes(&v);

        assert!(bitmap.get(32));
        assert!(bitmap.get(43));
        assert!(bitmap.get(47));

        assert_eq!(
            bitmap.as_bytes(),
            &[
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0001,
                0b1000_1000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
            ]
        );
    }

    #[test]
    fn bitmap_ref_msb() {
        let v = [
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0001,
            0b1000_1000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
        ];
        let bitmap = BitmapRef::<'_, MSB>::from_bytes(&v);

        assert!(bitmap.get(39));
        assert!(bitmap.get(40));
        assert!(bitmap.get(44));

        assert_eq!(
            bitmap.as_bytes(),
            &[
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0001,
                0b1000_1000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
            ]
        );
    }

    #[test]
    fn bitmap_ref_mut_lsb() {
        let mut v = [
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0001,
            0b1000_1000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
        ];
        let mut bitmap = BitmapRefMut::<'_, LSB>::from_bytes(&mut v);

        assert!(bitmap.get(32));
        assert!(bitmap.get(43));
        assert!(bitmap.get(47));

        bitmap.set(32, false);
        bitmap.set(43, false);
        bitmap.set(47, false);
        assert!(!bitmap.get(32));
        assert!(!bitmap.get(43));
        assert!(!bitmap.get(47));

        bitmap.set(0, true);
        assert_eq!(bitmap.as_bytes().len(), 10);
        assert!(bitmap.get(0));

        bitmap.set(15, true);
        assert_eq!(bitmap.as_bytes().len(), 10);
        assert!(bitmap.get(15));

        bitmap.set(24, true);
        assert_eq!(bitmap.as_bytes().len(), 10);
        assert!(bitmap.get(24));

        assert!(bitmap.try_set(132, true).is_err());
        assert_eq!(bitmap.as_bytes().len(), 10);
        assert!(!bitmap.get(132));

        assert_eq!(
            bitmap.as_bytes(),
            &[
                0b0000_0001,
                0b1000_0000,
                0b0000_0000,
                0b0000_0001,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
            ]
        );
    }

    #[test]
    fn bitmap_ref_mut_msb() {
        let mut v = [
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0001,
            0b1000_1000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
        ];
        let mut bitmap = BitmapRefMut::<'_, MSB>::from_bytes(&mut v);

        assert!(bitmap.get(39));
        assert!(bitmap.get(40));
        assert!(bitmap.get(44));

        bitmap.set(39, false);
        bitmap.set(40, false);
        bitmap.set(44, false);
        assert!(!bitmap.get(39));
        assert!(!bitmap.get(40));
        assert!(!bitmap.get(44));

        bitmap.set(0, true);
        assert_eq!(bitmap.as_bytes().len(), 10);
        assert!(bitmap.get(0));

        bitmap.set(15, true);
        assert_eq!(bitmap.as_bytes().len(), 10);
        assert!(bitmap.get(15));

        bitmap.set(24, true);
        assert_eq!(bitmap.as_bytes().len(), 10);
        assert!(bitmap.get(24));

        assert!(bitmap.try_set(132, true).is_err());
        assert_eq!(bitmap.as_bytes().len(), 10);
        assert!(!bitmap.get(132));

        assert_eq!(
            bitmap.as_bytes(),
            &[
                0b1000_0000,
                0b0000_0001,
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
            ]
        );
    }
}
