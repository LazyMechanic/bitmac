const BITS_IN_BYTE: usize = 8;

pub trait Extender {
    /// Will be called when the bitmap needs to extend its container.
    /// If the new size is smaller than actually needed, then the bitmap
    /// is extended to the minimum required bytes.
    fn extend(&self, old_len: usize, idx: usize) -> usize;
}

/// Extend to minimum required length.
///
/// Example:
/// ```
/// use bitmac::extender::{Extender, MinimumRequiredExtender};
/// let ext = MinimumRequiredExtender::default();
/// assert_eq!(ext.extend(0, 0), 1);
/// assert_eq!(ext.extend(0, 10), 2);
/// assert_eq!(ext.extend(0, 23), 3);
/// assert_eq!(ext.extend(3, 24), 4);
/// assert_eq!(ext.extend(3, 35), 5);
/// assert_eq!(ext.extend(3, 47), 6);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MinimumRequiredExtender;

impl Extender for MinimumRequiredExtender {
    fn extend(&self, old_len: usize, idx: usize) -> usize {
        old_len + (idx - old_len * BITS_IN_BYTE) / BITS_IN_BYTE + 1
    }
}

/// Extend to fixed length.
///
/// Example:
/// ```
/// use bitmac::extender::{Extender, FixedExtender};
/// let ext = FixedExtender(3);
/// assert_eq!(ext.extend(0, 0), 3);
/// assert_eq!(ext.extend(0, 10), 3);
/// assert_eq!(ext.extend(0, 23), 3);
/// assert_eq!(ext.extend(3, 24), 6);
/// assert_eq!(ext.extend(3, 35), 6);
/// assert_eq!(ext.extend(3, 47), 6);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FixedExtender(pub usize);

impl Extender for FixedExtender {
    fn extend(&self, old_len: usize, idx: usize) -> usize {
        let rest_required_bytes = (idx - old_len * BITS_IN_BYTE) / BITS_IN_BYTE + 1;
        let rest_fixed_bytes = usize::max(rest_required_bytes / self.0, 1) * self.0;
        old_len + rest_fixed_bytes
    }
}
