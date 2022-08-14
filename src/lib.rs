extern crate core;

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
