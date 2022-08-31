use crate::number::Number;

// Trait that provides functions for accessing single bit in number.
pub trait BitAccess: private::Sealed {
    /// Changes bit state.
    fn set<N>(num: N, bit_idx: usize, state: bool) -> N
    where
        N: Number;

    /// Gets bit state.
    fn get<N>(num: N, bit_idx: usize) -> bool
    where
        N: Number;
}

/// *Most Significant Bit* is a rule for bit accessing when 0th bit is the most significant bit (the last bit in order).
///
/// For example:
/// ```
/// use bitmac::{MSB, BitAccess};
/// assert_eq!(MSB::set(0b0000_0000u8, 0, true), 0b1000_0000u8);
/// ```
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct MSB;

impl BitAccess for MSB {
    fn set<N>(num: N, bit_idx: usize, state: bool) -> N
    where
        N: Number,
    {
        assert!(bit_idx < N::BITS_COUNT);

        let bit_idx = N::BITS_COUNT - bit_idx - 1;
        let mask = N::ONE << bit_idx;
        match state {
            true => num | mask,
            false => num & !mask,
        }
    }

    fn get<N>(num: N, bit_idx: usize) -> bool
    where
        N: Number,
    {
        assert!(bit_idx < N::BITS_COUNT);

        let bit_idx = N::BITS_COUNT - bit_idx - 1;
        num & (N::ONE << bit_idx) != N::ZERO
    }
}

/// *Least Significant Bit* is a rule for bit accessing when 0th bit is the least significant bit (the first bit in order).
///
/// For example:
/// ```
/// use bitmac::{LSB, BitAccess};
/// assert_eq!(LSB::set(0b0000_0000u8, 0, true), 0b0000_0001u8);
/// ```
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct LSB;

impl BitAccess for LSB {
    fn set<N>(num: N, bit_idx: usize, state: bool) -> N
    where
        N: Number,
    {
        assert!(bit_idx < N::BITS_COUNT);

        let mask = N::ONE << bit_idx;
        match state {
            true => num | mask,
            false => num & !mask,
        }
    }

    fn get<N>(num: N, bit_idx: usize) -> bool
    where
        N: Number,
    {
        assert!(bit_idx < N::BITS_COUNT);

        num & (N::ONE << bit_idx) != N::ZERO
    }
}

mod private {
    use crate::{LSB, MSB};

    pub trait Sealed {}

    impl Sealed for LSB {}
    impl Sealed for MSB {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msb_set() {
        assert_eq!(MSB::set(0b0000_0000u8, 0usize, true), 0b1000_0000u8);
        assert_eq!(MSB::set(0b0000_0000u8, 1usize, true), 0b0100_0000u8);
        assert_eq!(MSB::set(0b0000_0000u8, 2usize, true), 0b0010_0000u8);
        assert_eq!(MSB::set(0b0000_0000u8, 3usize, true), 0b0001_0000u8);
        assert_eq!(MSB::set(0b0000_0000u8, 4usize, true), 0b0000_1000u8);
        assert_eq!(MSB::set(0b0000_0000u8, 5usize, true), 0b0000_0100u8);
        assert_eq!(MSB::set(0b0000_0000u8, 6usize, true), 0b0000_0010u8);
        assert_eq!(MSB::set(0b0000_0000u8, 7usize, true), 0b0000_0001u8);

        assert_eq!(MSB::set(0b1111_1111u8, 0usize, false), 0b0111_1111u8);
        assert_eq!(MSB::set(0b1111_1111u8, 1usize, false), 0b1011_1111u8);
        assert_eq!(MSB::set(0b1111_1111u8, 2usize, false), 0b1101_1111u8);
        assert_eq!(MSB::set(0b1111_1111u8, 3usize, false), 0b1110_1111u8);
        assert_eq!(MSB::set(0b1111_1111u8, 4usize, false), 0b1111_0111u8);
        assert_eq!(MSB::set(0b1111_1111u8, 5usize, false), 0b1111_1011u8);
        assert_eq!(MSB::set(0b1111_1111u8, 6usize, false), 0b1111_1101u8);
        assert_eq!(MSB::set(0b1111_1111u8, 7usize, false), 0b1111_1110u8);
    }

    #[test]
    fn test_msb_get() {
        assert_eq!(MSB::get(0b0111_1111u8, 0usize), false);
        assert_eq!(MSB::get(0b1011_1111u8, 1usize), false);
        assert_eq!(MSB::get(0b1101_1111u8, 2usize), false);
        assert_eq!(MSB::get(0b1110_1111u8, 3usize), false);
        assert_eq!(MSB::get(0b1111_0111u8, 4usize), false);
        assert_eq!(MSB::get(0b1111_1011u8, 5usize), false);
        assert_eq!(MSB::get(0b1111_1101u8, 6usize), false);
        assert_eq!(MSB::get(0b1111_1110u8, 7usize), false);

        assert_eq!(MSB::get(0b1000_0000u8, 0usize), true);
        assert_eq!(MSB::get(0b0100_0000u8, 1usize), true);
        assert_eq!(MSB::get(0b0010_0000u8, 2usize), true);
        assert_eq!(MSB::get(0b0001_0000u8, 3usize), true);
        assert_eq!(MSB::get(0b0000_1000u8, 4usize), true);
        assert_eq!(MSB::get(0b0000_0100u8, 5usize), true);
        assert_eq!(MSB::get(0b0000_0010u8, 6usize), true);
        assert_eq!(MSB::get(0b0000_0001u8, 7usize), true);
    }

    #[test]
    fn test_lsb_set() {
        assert_eq!(LSB::set(0b0000_0000u8, 0usize, true), 0b0000_0001);
        assert_eq!(LSB::set(0b0000_0000u8, 1usize, true), 0b0000_0010);
        assert_eq!(LSB::set(0b0000_0000u8, 2usize, true), 0b0000_0100);
        assert_eq!(LSB::set(0b0000_0000u8, 3usize, true), 0b0000_1000);
        assert_eq!(LSB::set(0b0000_0000u8, 4usize, true), 0b0001_0000);
        assert_eq!(LSB::set(0b0000_0000u8, 5usize, true), 0b0010_0000);
        assert_eq!(LSB::set(0b0000_0000u8, 6usize, true), 0b0100_0000);
        assert_eq!(LSB::set(0b0000_0000u8, 7usize, true), 0b1000_0000);

        assert_eq!(LSB::set(0b1111_1111u8, 0usize, false), 0b1111_1110);
        assert_eq!(LSB::set(0b1111_1111u8, 1usize, false), 0b1111_1101);
        assert_eq!(LSB::set(0b1111_1111u8, 2usize, false), 0b1111_1011);
        assert_eq!(LSB::set(0b1111_1111u8, 3usize, false), 0b1111_0111);
        assert_eq!(LSB::set(0b1111_1111u8, 4usize, false), 0b1110_1111);
        assert_eq!(LSB::set(0b1111_1111u8, 5usize, false), 0b1101_1111);
        assert_eq!(LSB::set(0b1111_1111u8, 6usize, false), 0b1011_1111);
        assert_eq!(LSB::set(0b1111_1111u8, 7usize, false), 0b0111_1111);
    }

    #[test]
    fn test_lsb_get() {
        assert_eq!(LSB::get(0b1111_1110u8, 0usize), false);
        assert_eq!(LSB::get(0b1111_1101u8, 1usize), false);
        assert_eq!(LSB::get(0b1111_1011u8, 2usize), false);
        assert_eq!(LSB::get(0b1111_0111u8, 3usize), false);
        assert_eq!(LSB::get(0b1110_1111u8, 4usize), false);
        assert_eq!(LSB::get(0b1101_1111u8, 5usize), false);
        assert_eq!(LSB::get(0b1011_1111u8, 6usize), false);
        assert_eq!(LSB::get(0b0111_1111u8, 7usize), false);

        assert_eq!(LSB::get(0b0000_0001u8, 0usize), true);
        assert_eq!(LSB::get(0b0000_0010u8, 1usize), true);
        assert_eq!(LSB::get(0b0000_0100u8, 2usize), true);
        assert_eq!(LSB::get(0b0000_1000u8, 3usize), true);
        assert_eq!(LSB::get(0b0001_0000u8, 4usize), true);
        assert_eq!(LSB::get(0b0010_0000u8, 5usize), true);
        assert_eq!(LSB::get(0b0100_0000u8, 6usize), true);
        assert_eq!(LSB::get(0b1000_0000u8, 7usize), true);
    }
}
