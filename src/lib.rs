pub mod bit_order;
pub mod bitmap;
pub mod extender;

pub use bit_order::*;
pub use bitmap::*;
pub use extender::*;

const BITS_IN_BYTE: usize = 8;
