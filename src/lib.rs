//! # bitmac
//! This library provides implementation of bitmap with custom bit access,
//! a custom inner container and a variable or static container size.
//!
//! ## Features
//!
//! | Feature    | Description                                                                                                                                              |
//! |------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
//! | `bytes`    | to implement [`ContainerRead`] trait for [`Bytes`] and [`ContainerRead`], [`ContainerWrite`], [`Resizable`] and [`TryWithSlots`] traits for [`BytesMut`] |
//! | `smallvec` | to implement [`ContainerRead`], [`ContainerWrite`], [`Resizable`] and [`TryWithSlots`] traits for [`SmallVec`]                                           |
//!
//! ## BitAccess
//!
//! [`BitAccess`] is a trait that provides functions for accessing single bit in [`Number`].
//!
//! - [`LSB`]
//! - [`MSB`]
//!
//! ## ContainerRead & ContainerWrite
//!
//! [`ContainerRead`] and [`ContainerWrite`] are traits provides functions for accessing single slot ([`Number`]) in container.
//!
//! - `ContainerRead` for read-only access
//! - `ContainerWrite` for mutable access
//!
//! The traits are implemented for most standard types:
//! - `Vec<T>`
//! - `[T; N]`
//! - `&[T]`
//! - `&mut [T]`
//! - `T`
//! - `Bytes`
//! - `BytesMut`
//! - `SmallVec`
//!
//! You can implement them for your custom containers, the only one constraint is that containers should
//! consist of `Number`'s.
//!
//! ## StaticBitmap
//!
//! [`StaticBitmap`] is a bitmap that cannot be resized.
//!
//! Any structure that implements the [`ContainerRead`] (for read-only access) and [`ContainerWrite`]
//! (for mutable access) traits can be a container of bitmap (e.g. `[T; N]`, `&[T]`, `Vec<T>`, etc.).
//!
//! Usage example:
//! ```
//! # fn main() {
//! use bitmac::{StaticBitmap, LSB};
//!
//! // You can directly check every single bit
//! let bitmap = StaticBitmap::<_, LSB>::new([0b0000_0001u8, 0b0000_1000]);
//! assert!(bitmap.get(0));
//! assert!(bitmap.get(11));
//! assert!(!bitmap.get(13));
//! // Out of bounds bits always return false
//! assert!(!bitmap.get(128));
//!
//! // You can iterate over bits
//! let bitmap = StaticBitmap::<_, LSB>::new([0b0000_1001u8, 0b0000_1000]);
//! let mut iter = bitmap.iter().by_bits().enumerate();
//! assert_eq!(iter.next(), Some((0, true)));
//! assert_eq!(iter.next(), Some((1, false)));
//! assert_eq!(iter.next(), Some((2, false)));
//! assert_eq!(iter.next(), Some((3, true)));
//! assert_eq!(iter.next(), Some((4, false)));
//!
//! // You can check multiple bits at the same time through the intersection
//! use bitmac::Intersection;
//! let bitmap = StaticBitmap::<_, LSB>::new([0b0000_1001u8, 0b0000_1000]);
//! // .. by creating specific new container for result
//! let test = [0b0000_1001u8, 0b0000_0000];
//! assert_eq!(bitmap.intersection::<[u8; 2]>(&test), test);
//! // .. by using preallocated container for result
//! let test = [0b0000_1001u8, 0b0000_0000];
//! let mut result = [0u8; 2];
//! bitmap.intersection_in(&test, &mut result);
//! assert_eq!(result, test);
//! // .. by comparing length of difference that is equivalent to count of ones (bits) in result
//! let test = [0b0000_1001u8, 0b0000_0000];
//! assert_eq!(bitmap.intersection_len(&test), test.iter().fold(0, |acc, &v| acc + v.count_ones() as usize));
//!
//! // You can directly change every single bit
//! let mut bitmap = StaticBitmap::<_, LSB>::new([0b0000_1001u8, 0b0001_1000]);
//! assert!(bitmap.get(0));
//! assert!(bitmap.get(3));
//! assert!(bitmap.get(11));
//! assert!(bitmap.get(12));
//! assert!(!bitmap.get(13));
//! assert!(!bitmap.get(128));
//! bitmap.set(12, false);
//! assert!(!bitmap.get(12));
//! bitmap.set(13, true);
//! assert!(bitmap.get(13));
//! // Out of bounds bits return error
//! assert!(bitmap.try_set(128, true).is_err());
//! assert!(!bitmap.get(128));
//! # }
//! ```
//!
//! ## VarBitmap
//!
//! [`VarBitmap`] is a bitmap that can be resized by custom resizing strategy.
//!
//! Any structure that implements the [`ContainerRead`] (for read-only access) and
//! [`ContainerWrite`] + [`Resizable`] (for mutable access) traits can be a container of bitmap (e.g. `Vec<T>`).
//!
//! It has the same interface as `StaticBitmap` except that mutable access requires resizable container.
//! Container tries to grow if the changing bit is out of bounds.
//!
//! Usage example:
//! ```
//! # fn main() {
//! use bitmac::{VarBitmap, LSB, MinimumRequiredStrategy};
//!
//! // You can directly check every single bit
//! let bitmap = VarBitmap::<_, LSB, MinimumRequiredStrategy>::from_container(vec![0b0000_0001u8]);
//! assert!(bitmap.get(0));
//! assert!(!bitmap.get(11));
//! assert!(!bitmap.get(13));
//!
//! // You can iterate over bits
//! let bitmap = VarBitmap::<_, LSB, MinimumRequiredStrategy>::from_container(vec![0b0000_1001u8, 0b0000_1000]);
//! let mut iter = bitmap.iter().by_bits().enumerate();
//! assert_eq!(iter.next(), Some((0, true)));
//! assert_eq!(iter.next(), Some((1, false)));
//! assert_eq!(iter.next(), Some((2, false)));
//! assert_eq!(iter.next(), Some((3, true)));
//! assert_eq!(iter.next(), Some((4, false)));
//!
//! // You can check multiple bits at the same time through the intersection
//! use bitmac::Intersection;
//! let bitmap = VarBitmap::<_, LSB, MinimumRequiredStrategy>::from_container(vec![0b0000_1001u8, 0b0000_1000]);
//! // .. by creating specific new container for result
//! let test = [0b0000_1001u8, 0b0000_0000];
//! assert_eq!(bitmap.intersection::<[u8; 2]>(&test), test);
//! // .. by using preallocated container for result
//! let test = [0b0000_1001u8, 0b0000_0000];
//! let mut result = [0u8; 2];
//! bitmap.intersection_in(&test, &mut result);
//! assert_eq!(result, test);
//! // .. by comparing length of difference that is equivalent to count of ones (bits) in result
//! let test = [0b0000_1001u8, 0b0000_0000];
//! assert_eq!(bitmap.intersection_len(&test), test.iter().fold(0, |acc, &v| acc + v.count_ones() as usize));
//!
//! // You can directly change every bit
//! let mut bitmap = VarBitmap::<_, LSB, MinimumRequiredStrategy>::from_container(vec![0b0000_1001u8, 0b0001_1000]);
//! assert!(bitmap.get(0));
//! assert!(bitmap.get(3));
//! assert!(bitmap.get(11));
//! assert!(bitmap.get(12));
//! assert!(!bitmap.get(13));
//! assert!(!bitmap.get(128));
//! bitmap.set(12, false);
//! assert!(!bitmap.get(12));
//! bitmap.set(13, true);
//! assert!(bitmap.get(13));
//! // If you change the bit exceeding container's length and new bit state is `1` (`true`)
//! // then the container will automatically grow
//! bitmap.set(127, true);
//! assert!(bitmap.get(127));
//! assert_eq!(bitmap.as_ref().len(), 16);
//! # }
//! ```
//!
//! ### GrowStrategy
//!
//! [`GrowStrategy`] is a trait that controls how container will grow.
//! There are already several useful implemented strategies, but you can create your own.
//!
//! - [`MinimumRequiredStrategy`]
//! - [`FixedStrategy`]
//! - [`LimitStrategy`]
//! - [`ForceGrowStrategy`]
//!
//! [`GrowStrategy`]: crate::grow_strategy::GrowStrategy
//! [`MinimumRequiredStrategy`]: crate::grow_strategy::MinimumRequiredStrategy
//! [`FixedStrategy`]: crate::grow_strategy::FixedStrategy
//! [`LimitStrategy`]: crate::grow_strategy::LimitStrategy
//! [`ForceGrowStrategy`]: crate::grow_strategy::ForceGrowStrategy
//! [`BitAccess`]: crate::bit_access::BitAccess
//! [`LSB`]: crate::bit_access::LSB
//! [`MSB`]: crate::bit_access::MSB
//! [`ContainerRead`]: crate::container::ContainerRead
//! [`ContainerWrite`]: crate::container::ContainerWrite
//! [`Resizable`]: crate::resizable::Resizable
//! [`Number`]: crate::number::Number
//! [`TryWithSlots`]: crate::with_slots::TryWithSlots
//! [`VarBitmap`]: crate::var_bitmap::VarBitmap
//! [`StaticBitmap`]: crate::static_bitmap::StaticBitmap
//! [`Bytes`]: https://docs.rs/bytes/latest/bytes/
//! [`BytesMut`]: https://docs.rs/bytes/latest/bytes/
//! [`SmallVec`]: https://docs.rs/smallvec/latest/smallvec/

pub mod bit_access;
pub mod container;
pub mod error;
pub mod grow_strategy;
pub mod intersection;
pub mod iter;
pub mod number;
pub mod resizable;
pub mod static_bitmap;
pub mod union;
pub mod var_bitmap;
pub mod with_slots;

pub use bit_access::{BitAccess, LSB, MSB};
pub use error::{
    IntersectionError, OutOfBoundsError, ResizeError, SmallContainerSizeError, UnionError,
    WithSlotsError,
};
pub use grow_strategy::{FixedStrategy, LimitStrategy, MinimumRequiredStrategy};
pub use intersection::Intersection;
pub use static_bitmap::StaticBitmap;
pub use union::Union;
pub use var_bitmap::VarBitmap;
