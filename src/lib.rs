pub mod bit_order;
pub mod bitmap;
pub mod bitmap_ref;
pub mod extender;

pub use bit_order::*;
pub use bitmap::*;
pub use bitmap_ref::*;
pub use extender::*;

const BITS_IN_BYTE: usize = 8;
