// Trait provides functions for accessing to bit in byte.
pub trait BitAccess: private::Sealed {
    /// Changes bit state. `bit_idx` is guaranteed to be `0..=7`.
    fn set(&self, byte: u8, bit_idx: usize, state: bool) -> u8;

    /// Gets bit state. `bit_idx` is guaranteed to be `0..=7`.
    fn get(&self, byte: u8, bit_idx: usize) -> bool;
}

/// Most Significant Bit
///
/// Example:
/// ```
/// use bitmac::bit_access::{MSB, BitAccess};
/// assert_eq!(MSB.set(0b0000_0000, 0, true), 0b1000_0000);
/// ```
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct MSB;

impl BitAccess for MSB {
    fn set(&self, byte: u8, bit_idx: usize, state: bool) -> u8 {
        let mask = match bit_idx {
            0 => 0b1000_0000u8,
            1 => 0b0100_0000u8,
            2 => 0b0010_0000u8,
            3 => 0b0001_0000u8,
            4 => 0b0000_1000u8,
            5 => 0b0000_0100u8,
            6 => 0b0000_0010u8,
            7 => 0b0000_0001u8,
            _ => panic!("incorrect bit idx: {}, possible idx: [0..=7]", bit_idx),
        };

        match state {
            true => byte | mask,
            false => byte & !mask,
        }
    }

    fn get(&self, byte: u8, bit_idx: usize) -> bool {
        let offset = match bit_idx {
            0 => 7usize,
            1 => 6usize,
            2 => 5usize,
            3 => 4usize,
            4 => 3usize,
            5 => 2usize,
            6 => 1usize,
            7 => 0usize,
            _ => panic!("incorrect bit idx: {}, possible idx: [0..=7]", bit_idx),
        };

        (byte >> offset) & 0b0000_0001 == 0b0000_0001
    }
}

/// Least Significant Bit
///
/// Example:
/// ```
/// use bitmac::bit_access::{LSB, BitAccess};
/// assert_eq!(LSB.set(0b0000_0000, 0, true), 0b0000_0001);
/// ```
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct LSB;

impl BitAccess for LSB {
    fn set(&self, byte: u8, bit_idx: usize, state: bool) -> u8 {
        let mask = match bit_idx {
            0 => 0b0000_0001u8,
            1 => 0b0000_0010u8,
            2 => 0b0000_0100u8,
            3 => 0b0000_1000u8,
            4 => 0b0001_0000u8,
            5 => 0b0010_0000u8,
            6 => 0b0100_0000u8,
            7 => 0b1000_0000u8,
            _ => panic!("incorrect bit idx: {}, possible idx: [0..=7]", bit_idx),
        };

        match state {
            true => byte | mask,
            false => byte & !mask,
        }
    }

    fn get(&self, byte: u8, bit_idx: usize) -> bool {
        let offset = match bit_idx {
            0 => 0usize,
            1 => 1usize,
            2 => 2usize,
            3 => 3usize,
            4 => 4usize,
            5 => 5usize,
            6 => 6usize,
            7 => 7usize,
            _ => panic!("incorrect bit idx: {}, possible idx: [0..=7]", bit_idx),
        };

        (byte >> offset) & 0b0000_0001 == 0b0000_0001
    }
}

/// Bit accessor that is configured at runtime.
///
/// Example:
/// ```
/// use bitmac::bit_access::{MSB, LSB, DynBitAccess, BitAccess};
/// assert_eq!(DynBitAccess::MSB.set(0b0000_0000, 0, true), MSB.set(0b0000_0000, 0, true));
/// assert_eq!(DynBitAccess::LSB.set(0b0000_0000, 0, true), LSB.set(0b0000_0000, 0, true));
/// ```
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum DynBitAccess {
    MSB,
    LSB,
}

impl BitAccess for DynBitAccess {
    fn set(&self, byte: u8, bit_idx: usize, state: bool) -> u8 {
        match self {
            DynBitAccess::MSB => MSB.set(byte, bit_idx, state),
            DynBitAccess::LSB => LSB.set(byte, bit_idx, state),
        }
    }

    fn get(&self, byte: u8, bit_idx: usize) -> bool {
        match self {
            DynBitAccess::MSB => MSB.get(byte, bit_idx),
            DynBitAccess::LSB => LSB.get(byte, bit_idx),
        }
    }
}

mod private {
    use super::{DynBitAccess, LSB, MSB};

    pub trait Sealed {}

    impl Sealed for LSB {}
    impl Sealed for MSB {}
    impl Sealed for DynBitAccess {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msb_set() {
        let a = MSB;

        assert_eq!(a.set(0b0000_0000u8, 0usize, true), 0b1000_0000u8);
        assert_eq!(a.set(0b0000_0000u8, 1usize, true), 0b0100_0000u8);
        assert_eq!(a.set(0b0000_0000u8, 2usize, true), 0b0010_0000u8);
        assert_eq!(a.set(0b0000_0000u8, 3usize, true), 0b0001_0000u8);
        assert_eq!(a.set(0b0000_0000u8, 4usize, true), 0b0000_1000u8);
        assert_eq!(a.set(0b0000_0000u8, 5usize, true), 0b0000_0100u8);
        assert_eq!(a.set(0b0000_0000u8, 6usize, true), 0b0000_0010u8);
        assert_eq!(a.set(0b0000_0000u8, 7usize, true), 0b0000_0001u8);

        assert_eq!(a.set(0b1111_1111u8, 0usize, false), 0b0111_1111u8);
        assert_eq!(a.set(0b1111_1111u8, 1usize, false), 0b1011_1111u8);
        assert_eq!(a.set(0b1111_1111u8, 2usize, false), 0b1101_1111u8);
        assert_eq!(a.set(0b1111_1111u8, 3usize, false), 0b1110_1111u8);
        assert_eq!(a.set(0b1111_1111u8, 4usize, false), 0b1111_0111u8);
        assert_eq!(a.set(0b1111_1111u8, 5usize, false), 0b1111_1011u8);
        assert_eq!(a.set(0b1111_1111u8, 6usize, false), 0b1111_1101u8);
        assert_eq!(a.set(0b1111_1111u8, 7usize, false), 0b1111_1110u8);
    }

    #[test]
    fn test_msb_get() {
        let a = MSB;

        assert_eq!(a.get(0b0111_1111u8, 0usize), false);
        assert_eq!(a.get(0b1011_1111u8, 1usize), false);
        assert_eq!(a.get(0b1101_1111u8, 2usize), false);
        assert_eq!(a.get(0b1110_1111u8, 3usize), false);
        assert_eq!(a.get(0b1111_0111u8, 4usize), false);
        assert_eq!(a.get(0b1111_1011u8, 5usize), false);
        assert_eq!(a.get(0b1111_1101u8, 6usize), false);
        assert_eq!(a.get(0b1111_1110u8, 7usize), false);

        assert_eq!(a.get(0b1000_0000u8, 0usize), true);
        assert_eq!(a.get(0b0100_0000u8, 1usize), true);
        assert_eq!(a.get(0b0010_0000u8, 2usize), true);
        assert_eq!(a.get(0b0001_0000u8, 3usize), true);
        assert_eq!(a.get(0b0000_1000u8, 4usize), true);
        assert_eq!(a.get(0b0000_0100u8, 5usize), true);
        assert_eq!(a.get(0b0000_0010u8, 6usize), true);
        assert_eq!(a.get(0b0000_0001u8, 7usize), true);
    }

    #[test]
    fn test_lsb_set() {
        let a = LSB;

        assert_eq!(a.set(0b0000_0000u8, 0usize, true), 0b0000_0001);
        assert_eq!(a.set(0b0000_0000u8, 1usize, true), 0b0000_0010);
        assert_eq!(a.set(0b0000_0000u8, 2usize, true), 0b0000_0100);
        assert_eq!(a.set(0b0000_0000u8, 3usize, true), 0b0000_1000);
        assert_eq!(a.set(0b0000_0000u8, 4usize, true), 0b0001_0000);
        assert_eq!(a.set(0b0000_0000u8, 5usize, true), 0b0010_0000);
        assert_eq!(a.set(0b0000_0000u8, 6usize, true), 0b0100_0000);
        assert_eq!(a.set(0b0000_0000u8, 7usize, true), 0b1000_0000);

        assert_eq!(a.set(0b1111_1111u8, 0usize, false), 0b1111_1110);
        assert_eq!(a.set(0b1111_1111u8, 1usize, false), 0b1111_1101);
        assert_eq!(a.set(0b1111_1111u8, 2usize, false), 0b1111_1011);
        assert_eq!(a.set(0b1111_1111u8, 3usize, false), 0b1111_0111);
        assert_eq!(a.set(0b1111_1111u8, 4usize, false), 0b1110_1111);
        assert_eq!(a.set(0b1111_1111u8, 5usize, false), 0b1101_1111);
        assert_eq!(a.set(0b1111_1111u8, 6usize, false), 0b1011_1111);
        assert_eq!(a.set(0b1111_1111u8, 7usize, false), 0b0111_1111);
    }

    #[test]
    fn test_lsb_get() {
        let a = LSB;

        assert_eq!(a.get(0b1111_1110u8, 0usize), false);
        assert_eq!(a.get(0b1111_1101u8, 1usize), false);
        assert_eq!(a.get(0b1111_1011u8, 2usize), false);
        assert_eq!(a.get(0b1111_0111u8, 3usize), false);
        assert_eq!(a.get(0b1110_1111u8, 4usize), false);
        assert_eq!(a.get(0b1101_1111u8, 5usize), false);
        assert_eq!(a.get(0b1011_1111u8, 6usize), false);
        assert_eq!(a.get(0b0111_1111u8, 7usize), false);

        assert_eq!(a.get(0b0000_0001u8, 0usize), true);
        assert_eq!(a.get(0b0000_0010u8, 1usize), true);
        assert_eq!(a.get(0b0000_0100u8, 2usize), true);
        assert_eq!(a.get(0b0000_1000u8, 3usize), true);
        assert_eq!(a.get(0b0001_0000u8, 4usize), true);
        assert_eq!(a.get(0b0010_0000u8, 5usize), true);
        assert_eq!(a.get(0b0100_0000u8, 6usize), true);
        assert_eq!(a.get(0b1000_0000u8, 7usize), true);
    }

    #[test]
    fn test_dyn_set() {
        let a = DynBitAccess::LSB;

        assert_eq!(a.set(0b0000_0000u8, 0usize, true), 0b0000_0001);
        assert_eq!(a.set(0b0000_0000u8, 1usize, true), 0b0000_0010);
        assert_eq!(a.set(0b0000_0000u8, 2usize, true), 0b0000_0100);
        assert_eq!(a.set(0b0000_0000u8, 3usize, true), 0b0000_1000);
        assert_eq!(a.set(0b0000_0000u8, 4usize, true), 0b0001_0000);
        assert_eq!(a.set(0b0000_0000u8, 5usize, true), 0b0010_0000);
        assert_eq!(a.set(0b0000_0000u8, 6usize, true), 0b0100_0000);
        assert_eq!(a.set(0b0000_0000u8, 7usize, true), 0b1000_0000);

        assert_eq!(a.set(0b1111_1111u8, 0usize, false), 0b1111_1110);
        assert_eq!(a.set(0b1111_1111u8, 1usize, false), 0b1111_1101);
        assert_eq!(a.set(0b1111_1111u8, 2usize, false), 0b1111_1011);
        assert_eq!(a.set(0b1111_1111u8, 3usize, false), 0b1111_0111);
        assert_eq!(a.set(0b1111_1111u8, 4usize, false), 0b1110_1111);
        assert_eq!(a.set(0b1111_1111u8, 5usize, false), 0b1101_1111);
        assert_eq!(a.set(0b1111_1111u8, 6usize, false), 0b1011_1111);
        assert_eq!(a.set(0b1111_1111u8, 7usize, false), 0b0111_1111);

        let a = DynBitAccess::MSB;

        assert_eq!(a.set(0b0000_0000u8, 0usize, true), 0b1000_0000u8);
        assert_eq!(a.set(0b0000_0000u8, 1usize, true), 0b0100_0000u8);
        assert_eq!(a.set(0b0000_0000u8, 2usize, true), 0b0010_0000u8);
        assert_eq!(a.set(0b0000_0000u8, 3usize, true), 0b0001_0000u8);
        assert_eq!(a.set(0b0000_0000u8, 4usize, true), 0b0000_1000u8);
        assert_eq!(a.set(0b0000_0000u8, 5usize, true), 0b0000_0100u8);
        assert_eq!(a.set(0b0000_0000u8, 6usize, true), 0b0000_0010u8);
        assert_eq!(a.set(0b0000_0000u8, 7usize, true), 0b0000_0001u8);

        assert_eq!(a.set(0b1111_1111u8, 0usize, false), 0b0111_1111u8);
        assert_eq!(a.set(0b1111_1111u8, 1usize, false), 0b1011_1111u8);
        assert_eq!(a.set(0b1111_1111u8, 2usize, false), 0b1101_1111u8);
        assert_eq!(a.set(0b1111_1111u8, 3usize, false), 0b1110_1111u8);
        assert_eq!(a.set(0b1111_1111u8, 4usize, false), 0b1111_0111u8);
        assert_eq!(a.set(0b1111_1111u8, 5usize, false), 0b1111_1011u8);
        assert_eq!(a.set(0b1111_1111u8, 6usize, false), 0b1111_1101u8);
        assert_eq!(a.set(0b1111_1111u8, 7usize, false), 0b1111_1110u8);
    }

    #[test]
    fn test_dyn_get() {
        let a = DynBitAccess::LSB;

        assert_eq!(a.get(0b1111_1110u8, 0usize), false);
        assert_eq!(a.get(0b1111_1101u8, 1usize), false);
        assert_eq!(a.get(0b1111_1011u8, 2usize), false);
        assert_eq!(a.get(0b1111_0111u8, 3usize), false);
        assert_eq!(a.get(0b1110_1111u8, 4usize), false);
        assert_eq!(a.get(0b1101_1111u8, 5usize), false);
        assert_eq!(a.get(0b1011_1111u8, 6usize), false);
        assert_eq!(a.get(0b0111_1111u8, 7usize), false);

        assert_eq!(a.get(0b0000_0001u8, 0usize), true);
        assert_eq!(a.get(0b0000_0010u8, 1usize), true);
        assert_eq!(a.get(0b0000_0100u8, 2usize), true);
        assert_eq!(a.get(0b0000_1000u8, 3usize), true);
        assert_eq!(a.get(0b0001_0000u8, 4usize), true);
        assert_eq!(a.get(0b0010_0000u8, 5usize), true);
        assert_eq!(a.get(0b0100_0000u8, 6usize), true);
        assert_eq!(a.get(0b1000_0000u8, 7usize), true);

        let a = DynBitAccess::MSB;

        assert_eq!(a.get(0b0111_1111u8, 0usize), false);
        assert_eq!(a.get(0b1011_1111u8, 1usize), false);
        assert_eq!(a.get(0b1101_1111u8, 2usize), false);
        assert_eq!(a.get(0b1110_1111u8, 3usize), false);
        assert_eq!(a.get(0b1111_0111u8, 4usize), false);
        assert_eq!(a.get(0b1111_1011u8, 5usize), false);
        assert_eq!(a.get(0b1111_1101u8, 6usize), false);
        assert_eq!(a.get(0b1111_1110u8, 7usize), false);

        assert_eq!(a.get(0b1000_0000u8, 0usize), true);
        assert_eq!(a.get(0b0100_0000u8, 1usize), true);
        assert_eq!(a.get(0b0010_0000u8, 2usize), true);
        assert_eq!(a.get(0b0001_0000u8, 3usize), true);
        assert_eq!(a.get(0b0000_1000u8, 4usize), true);
        assert_eq!(a.get(0b0000_0100u8, 5usize), true);
        assert_eq!(a.get(0b0000_0010u8, 6usize), true);
        assert_eq!(a.get(0b0000_0001u8, 7usize), true);
    }
}
