#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BitOrder {
    /// Most Significant Bit
    ///
    /// Example:
    /// ```
    /// # use bitmac::bit_order::BitOrder;
    /// assert_eq!(BitOrder::MSB.set(0b0000_0000, 0, true), 0b1000_0000);
    /// ```
    MSB,
    /// Least Significant Bit
    ///
    /// Example:
    /// ```
    /// # use bitmac::bit_order::BitOrder;
    /// assert_eq!(BitOrder::LSB.set(0b0000_0000, 0, true), 0b0000_0001);
    /// ```
    LSB,
}

impl BitOrder {
    pub fn set(&self, byte: u8, bit_idx: usize, state: bool) -> u8 {
        let mask = match self {
            BitOrder::MSB => match bit_idx {
                0 => 0b1000_0000u8,
                1 => 0b0100_0000u8,
                2 => 0b0010_0000u8,
                3 => 0b0001_0000u8,
                4 => 0b0000_1000u8,
                5 => 0b0000_0100u8,
                6 => 0b0000_0010u8,
                7 => 0b0000_0001u8,
                _ => panic!("incorrect bit idx: {}, possible idx: [0..=7]", bit_idx),
            },
            BitOrder::LSB => match bit_idx {
                0 => 0b0000_0001u8,
                1 => 0b0000_0010u8,
                2 => 0b0000_0100u8,
                3 => 0b0000_1000u8,
                4 => 0b0001_0000u8,
                5 => 0b0010_0000u8,
                6 => 0b0100_0000u8,
                7 => 0b1000_0000u8,
                _ => panic!("incorrect bit idx: {}, possible idx: [0..=7]", bit_idx),
            },
        };

        match state {
            true => byte | mask,
            false => byte & !mask,
        }
    }

    pub fn get(&self, byte: u8, bit_idx: usize) -> bool {
        let offset = match self {
            BitOrder::MSB => match bit_idx {
                0 => 7usize,
                1 => 6usize,
                2 => 5usize,
                3 => 4usize,
                4 => 3usize,
                5 => 2usize,
                6 => 1usize,
                7 => 0usize,
                _ => panic!("incorrect bit idx: {}, possible idx: [0..=7]", bit_idx),
            },
            BitOrder::LSB => match bit_idx {
                0 => 0usize,
                1 => 1usize,
                2 => 2usize,
                3 => 3usize,
                4 => 4usize,
                5 => 5usize,
                6 => 6usize,
                7 => 7usize,
                _ => panic!("incorrect bit idx: {}, possible idx: [0..=7]", bit_idx),
            },
        };

        (byte >> offset) & 0b0000_0001 == 0b0000_0001
    }
}

impl Default for BitOrder {
    fn default() -> Self {
        BitOrder::LSB
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msb_set() {
        let order = BitOrder::MSB;

        assert_eq!(order.set(0b0000_0000u8, 0usize, true), 0b1000_0000u8);
        assert_eq!(order.set(0b0000_0000u8, 1usize, true), 0b0100_0000u8);
        assert_eq!(order.set(0b0000_0000u8, 2usize, true), 0b0010_0000u8);
        assert_eq!(order.set(0b0000_0000u8, 3usize, true), 0b0001_0000u8);
        assert_eq!(order.set(0b0000_0000u8, 4usize, true), 0b0000_1000u8);
        assert_eq!(order.set(0b0000_0000u8, 5usize, true), 0b0000_0100u8);
        assert_eq!(order.set(0b0000_0000u8, 6usize, true), 0b0000_0010u8);
        assert_eq!(order.set(0b0000_0000u8, 7usize, true), 0b0000_0001u8);

        assert_eq!(order.set(0b1111_1111u8, 0usize, false), 0b0111_1111u8);
        assert_eq!(order.set(0b1111_1111u8, 1usize, false), 0b1011_1111u8);
        assert_eq!(order.set(0b1111_1111u8, 2usize, false), 0b1101_1111u8);
        assert_eq!(order.set(0b1111_1111u8, 3usize, false), 0b1110_1111u8);
        assert_eq!(order.set(0b1111_1111u8, 4usize, false), 0b1111_0111u8);
        assert_eq!(order.set(0b1111_1111u8, 5usize, false), 0b1111_1011u8);
        assert_eq!(order.set(0b1111_1111u8, 6usize, false), 0b1111_1101u8);
        assert_eq!(order.set(0b1111_1111u8, 7usize, false), 0b1111_1110u8);
    }

    #[test]
    fn test_msb_get() {
        let order = BitOrder::MSB;

        assert_eq!(order.get(0b0111_1111u8, 0usize), false);
        assert_eq!(order.get(0b1011_1111u8, 1usize), false);
        assert_eq!(order.get(0b1101_1111u8, 2usize), false);
        assert_eq!(order.get(0b1110_1111u8, 3usize), false);
        assert_eq!(order.get(0b1111_0111u8, 4usize), false);
        assert_eq!(order.get(0b1111_1011u8, 5usize), false);
        assert_eq!(order.get(0b1111_1101u8, 6usize), false);
        assert_eq!(order.get(0b1111_1110u8, 7usize), false);

        assert_eq!(order.get(0b1000_0000u8, 0usize), true);
        assert_eq!(order.get(0b0100_0000u8, 1usize), true);
        assert_eq!(order.get(0b0010_0000u8, 2usize), true);
        assert_eq!(order.get(0b0001_0000u8, 3usize), true);
        assert_eq!(order.get(0b0000_1000u8, 4usize), true);
        assert_eq!(order.get(0b0000_0100u8, 5usize), true);
        assert_eq!(order.get(0b0000_0010u8, 6usize), true);
        assert_eq!(order.get(0b0000_0001u8, 7usize), true);
    }

    #[test]
    fn test_lsb_set() {
        let order = BitOrder::LSB;

        assert_eq!(order.set(0b0000_0000u8, 0usize, true), 0b0000_0001);
        assert_eq!(order.set(0b0000_0000u8, 1usize, true), 0b0000_0010);
        assert_eq!(order.set(0b0000_0000u8, 2usize, true), 0b0000_0100);
        assert_eq!(order.set(0b0000_0000u8, 3usize, true), 0b0000_1000);
        assert_eq!(order.set(0b0000_0000u8, 4usize, true), 0b0001_0000);
        assert_eq!(order.set(0b0000_0000u8, 5usize, true), 0b0010_0000);
        assert_eq!(order.set(0b0000_0000u8, 6usize, true), 0b0100_0000);
        assert_eq!(order.set(0b0000_0000u8, 7usize, true), 0b1000_0000);

        assert_eq!(order.set(0b1111_1111u8, 0usize, false), 0b1111_1110);
        assert_eq!(order.set(0b1111_1111u8, 1usize, false), 0b1111_1101);
        assert_eq!(order.set(0b1111_1111u8, 2usize, false), 0b1111_1011);
        assert_eq!(order.set(0b1111_1111u8, 3usize, false), 0b1111_0111);
        assert_eq!(order.set(0b1111_1111u8, 4usize, false), 0b1110_1111);
        assert_eq!(order.set(0b1111_1111u8, 5usize, false), 0b1101_1111);
        assert_eq!(order.set(0b1111_1111u8, 6usize, false), 0b1011_1111);
        assert_eq!(order.set(0b1111_1111u8, 7usize, false), 0b0111_1111);
    }

    #[test]
    fn test_lsb_get() {
        let order = BitOrder::LSB;

        assert_eq!(order.get(0b1111_1110u8, 0usize), false);
        assert_eq!(order.get(0b1111_1101u8, 1usize), false);
        assert_eq!(order.get(0b1111_1011u8, 2usize), false);
        assert_eq!(order.get(0b1111_0111u8, 3usize), false);
        assert_eq!(order.get(0b1110_1111u8, 4usize), false);
        assert_eq!(order.get(0b1101_1111u8, 5usize), false);
        assert_eq!(order.get(0b1011_1111u8, 6usize), false);
        assert_eq!(order.get(0b0111_1111u8, 7usize), false);

        assert_eq!(order.get(0b0000_0001u8, 0usize), true);
        assert_eq!(order.get(0b0000_0010u8, 1usize), true);
        assert_eq!(order.get(0b0000_0100u8, 2usize), true);
        assert_eq!(order.get(0b0000_1000u8, 3usize), true);
        assert_eq!(order.get(0b0001_0000u8, 4usize), true);
        assert_eq!(order.get(0b0010_0000u8, 5usize), true);
        assert_eq!(order.get(0b0100_0000u8, 6usize), true);
        assert_eq!(order.get(0b1000_0000u8, 7usize), true);
    }
}
