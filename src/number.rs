use std::{
    fmt::Binary,
    ops::{Add, BitAnd, BitOr, Not, Shl, Shr, Sub},
};

pub trait Number:
    Copy
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Not<Output = Self>
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
    + Eq
    + Ord
    + Binary
{
    const BITS_COUNT: usize;
    const BYTES_COUNT: usize;
    const ONE: Self;
    const ZERO: Self;
    const MAX: Self;
    const MIN: Self;
    const BYTE_MASK: Self;
}

macro_rules! number_impl {
    ($ty:ty, $bits:literal) => {
        impl Number for $ty {
            const BITS_COUNT: usize = $bits;
            const BYTES_COUNT: usize = $bits / 8;
            const ONE: Self = 1;
            const ZERO: Self = 0;
            const MAX: Self = <$ty>::MAX;
            const MIN: Self = <$ty>::MIN;
            const BYTE_MASK: Self = 0b1111_1111;
        }
    };
}

number_impl!(u8, 8);
number_impl!(u16, 16);
number_impl!(u32, 32);
number_impl!(u64, 64);
number_impl!(u128, 128);
