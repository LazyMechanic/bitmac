//! # bitmac
//! This library provides implementation of bitmap with custom bit accessing and resizing strategy.
//!
//! ### Features
//! - `bytes` - implemented trait [`ContainerMut`] for [`BytesMut`]
//! - `smallvec` - implemented trait [`ContainerMut`] for [`SmallVec`]
//!
//! ### Resizing strategy
//! The library provides several resizing strategy.
//!
//! #### MinimalRequiredStrategy
//! Strategy [`MinimumRequiredStrategy`] resizes to minimum required bytes.
//! ```
//! # use bitmac::{MinimumRequiredStrategy, Bitmap, LSB};
//! # fn main() {
//! let mut bitmap = Bitmap::<Vec<u8>, MinimumRequiredStrategy, LSB>::default();
//!
//! bitmap.set(0, true);
//! assert_eq!(bitmap.as_bytes().len(), 1);
//!
//! bitmap.set(15, true);
//! assert_eq!(bitmap.as_bytes().len(), 2);
//! # }
//! ```
//!
//! #### FixedStrategy
//! Strategy [`FixedStrategy`] advances size by fixed steps.
//!
//! ```
//! # use bitmac::{FixedStrategy, Bitmap, LSB};
//! # fn main() {
//! let mut bitmap = Bitmap::<Vec<u8>, _, LSB>::with_resizing_strategy(FixedStrategy(3));
//!
//! bitmap.set(0, true);
//! assert_eq!(bitmap.as_bytes().len(), 3);
//!
//! bitmap.set(15, true);
//! assert_eq!(bitmap.as_bytes().len(), 3);
//!
//! bitmap.set(24, true);
//! assert_eq!(bitmap.as_bytes().len(), 6);
//! # }
//! ```
//!
//! #### StaticStrategy
//! Strategy [`StaticStrategy`] never increases the size, returns an error if an increase is required.
//!
//! Useful for const containers (`[u8; N]`).
//!
//! ```
//! # use bitmac::{StaticStrategy, Bitmap, LSB};
//! # fn main() {
//! let mut bitmap = Bitmap::<_, _, LSB>::new([0u8; 2], StaticStrategy);
//!
//! bitmap.set(0, true);
//! assert_eq!(bitmap.as_bytes().len(), 2);
//!
//! bitmap.set(15, true);
//! assert_eq!(bitmap.as_bytes().len(), 2);
//!
//! assert!(bitmap.try_set(24, true).is_err());
//! assert_eq!(bitmap.as_bytes().len(), 2);
//! # }
//! ```
//!
//! You can implement your own [`ResizingStrategy`].
//!
//! ### BitAccess
//! Trait [`BitAccess`] provides functions for accessing to bit in byte.
//! The bytes in a bitmap can be stored in [`LSB`] or [`MSB`] order.
//!
//! #### LSB
//! In [`LSB`] order, the 0th bit of the bitmap is the least significant bit.
//!
//! For example:
//! ```
//! # use bitmac::{StaticStrategy, Bitmap, LSB};
//! # fn main() {
//! let bitmap = Bitmap::<_, _, LSB>::new([0b0000_0001], StaticStrategy);
//! assert!(bitmap.get(0));
//! # }
//! ```
//!
//! #### MSB
//! In [`MSB`] order, the 0th bit of the bitmap is the most significant bit.
//!
//! For example:
//! ```
//! # use bitmac::{StaticStrategy, Bitmap, MSB};
//! # fn main() {
//! let bitmap = Bitmap::<_, _, MSB>::new([0b0000_0001], StaticStrategy);
//! assert!(bitmap.get(7));
//! # }
//! ```
//!
//! #### DynBitAccess
//! [`DynBitAccess`] can be configured in runtime ([`LSB`] or [`MSB`])
//!
//! For example:
//! ```
//! # use bitmac::{StaticStrategy, Bitmap, DynBitAccess};
//! # fn main() {
//! let bitmap = Bitmap::<_, _, DynBitAccess>::from_parts([0b0000_0001], StaticStrategy, DynBitAccess::LSB);
//! assert!(bitmap.get(0));
//!
//! let bitmap = Bitmap::<_, _, DynBitAccess>::from_parts([0b0000_0001], StaticStrategy, DynBitAccess::MSB);
//! assert!(bitmap.get(7));
//! # }
//! ```
//!
//! ### Bitmap
//! [`Bitmap`] that owns the container. Container can dynamically grow if needed.
//!
//! You can use any container that implements the `AsRef<[u8]>` trait for read-only access and
//! the [`ContainerMut`] trait for write access
//!
//! Usage example:
//! ```
//! # use bitmac::{MinimumRequiredStrategy, LSB, Bitmap};
//! # fn main() {
//! let mut bitmap = Bitmap::<Vec<u8>, MinimumRequiredStrategy, LSB>::default();
//!
//! assert_eq!(bitmap.as_bytes().len(), 0);
//!
//! assert!(!bitmap.get(0));
//! assert!(!bitmap.get(7));
//! assert!(!bitmap.get(300));
//!
//! bitmap.set(15, true);
//! assert_eq!(bitmap.as_bytes().len(), 2);
//! assert!(bitmap.get(15));
//! # }
//! ```
//!
//! ### BitmapRef
//! [`BitmapRef`] is bitmap that borrows bytes.
//!
//! Helpful if you have already allocated bytes and you want to just look at them as bitmap,
//! without modifications.
//!
//! Usage example:
//! ```
//! # use bitmac::{MinimumRequiredStrategy, LSB, BitmapRef};
//! # fn main() {
//! let bitmap = BitmapRef::<'_, LSB>::from_bytes(&[0b0000_1000, 0b0000_0001]);
//!
//! assert_eq!(bitmap.get(3), true);
//! assert_eq!(bitmap.get(8), true);
//!
//! assert_eq!(bitmap.get(1), false);
//! assert_eq!(bitmap.get(7), false);
//! assert_eq!(bitmap.get(300), false);
//!
//! assert_eq!(bitmap.as_bytes().len(), 2);
//! # }
//! ```
//!
//! ### BitmapRefMut
//! [`BitmapRefMut`] is bitmap that borrows mutable bytes.
//!
//! Helpful if you have already allocated bytes and you want to just look at them as bitmap
//! and modify it. Cannot increase the number of bytes.
//!
//! Usage example:
//! ```
//! # use bitmac::{MinimumRequiredStrategy, LSB, BitmapRef};
//! # fn main() {
//! let bitmap = BitmapRef::<'_, LSB>::from_bytes(&[0b0000_1000, 0b0000_0001]);
//!
//! assert_eq!(bitmap.get(3), true);
//! assert_eq!(bitmap.get(8), true);
//!
//! assert_eq!(bitmap.get(1), false);
//! assert_eq!(bitmap.get(7), false);
//! assert_eq!(bitmap.get(300), false);
//!
//! assert_eq!(bitmap.as_bytes().len(), 2);
//! # }
//! ```
//!
//! [`ResizingStrategy`]: crate::resizing_strategy::ResizingStrategy
//! [`MinimumRequiredStrategy`]: crate::resizing_strategy::MinimumRequiredStrategy
//! [`FixedStrategy`]: crate::resizing_strategy::FixedStrategy
//! [`StaticStrategy`]: crate::resizing_strategy::StaticStrategy
//! [`BitAccess`]: crate::bit_access::BitAccess
//! [`LSB`]: crate::bit_access::LSB
//! [`MSB`]: crate::bit_access::MSB
//! [`DynBitAccess`]: crate::bit_access::DynBitAccess
//! [`ContainerMut`]: crate::container::ContainerMut
//! [`Bitmap`]: crate::bitmap::Bitmap
//! [`BitmapRef`]: crate::bitmap_ref::BitmapRef
//! [`BitmapRefMut`]: crate::bitmap_ref::BitmapRefMut
//! [`BytesMut`]: https://docs.rs/bytes/latest/bytes/
//! [`SmallVec`]: https://docs.rs/smallvec/latest/smallvec/

pub mod bit_access;
pub mod bitmap;
pub mod bitmap_ref;
pub mod container;
pub mod error;
pub mod resizing_strategy;

pub use bit_access::*;
pub use bitmap::*;
pub use bitmap_ref::*;
pub use container::*;
pub use error::*;
pub use resizing_strategy::*;

const BITS_IN_BYTE: usize = 8;

fn set_impl<B>(data: &mut [u8], ba: &B, idx: usize, v: bool)
where
    B: BitAccess,
{
    let bit_idx = idx & 0b0111;
    let byte_idx = idx >> 3;

    let byte = &mut data[byte_idx];
    *byte = ba.set(*byte, bit_idx, v);
}

fn get_impl<B>(data: &[u8], ba: &B, idx: usize) -> bool
where
    B: BitAccess,
{
    let bit_idx = idx & 0b0111;
    let byte_idx = idx >> 3;

    // If idx out of bounds
    if byte_idx >= data.len() {
        return false;
    }

    ba.get(data[byte_idx], bit_idx)
}
