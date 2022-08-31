use std::marker::PhantomData;

use crate::{container::ContainerRead, number::Number, BitAccess};

/// An iterator over slots that moves out of a container.
pub struct IntoIter<D, B> {
    slot_idx: usize,
    data: D,
    phantom: PhantomData<B>,
}

impl<D, B> IntoIter<D, B> {
    pub(crate) fn new(data: D) -> Self {
        Self {
            slot_idx: 0,
            data,
            phantom: Default::default(),
        }
    }

    pub fn by_bits(self) -> IntoIterBits<D, B> {
        IntoIterBits {
            slot_idx: self.slot_idx,
            bit_idx: 0,
            data: self.data,
            phantom: Default::default(),
        }
    }
}

impl<D, B> Iterator for IntoIter<D, B>
where
    D: ContainerRead<B>,
    B: BitAccess,
{
    type Item = D::Slot;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slot_idx < self.data.slots_count() {
            let v = self.data.get_slot(self.slot_idx);
            self.slot_idx += 1;
            Some(v)
        } else {
            None
        }
    }
}

/// An iterator over bits that moves out of a container.
pub struct IntoIterBits<D, B> {
    slot_idx: usize,
    bit_idx: usize,
    data: D,
    phantom: PhantomData<B>,
}

impl<D, B, N> Iterator for IntoIterBits<D, B>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
    N: Number,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slot_idx < self.data.slots_count() {
            let slot = self.data.get_slot(self.slot_idx);
            let v = B::get(slot, self.bit_idx);

            self.bit_idx = (self.bit_idx + 1) % N::BITS_COUNT;
            if self.bit_idx == 0 {
                self.slot_idx += 1;
            }

            Some(v)
        } else {
            None
        }
    }
}

/// An iterator over slots.
pub struct Iter<'a, D, B> {
    slot_idx: usize,
    data: &'a D,
    phantom: PhantomData<B>,
}

impl<'a, D, B> Iter<'a, D, B> {
    pub(crate) fn new(data: &'a D) -> Self {
        Self {
            slot_idx: 0,
            data,
            phantom: Default::default(),
        }
    }

    pub fn by_bits(self) -> IterBits<'a, D, B> {
        IterBits {
            slot_idx: self.slot_idx,
            bit_idx: 0,
            data: self.data,
            phantom: Default::default(),
        }
    }
}

impl<D, B> Iterator for Iter<'_, D, B>
where
    D: ContainerRead<B>,
    B: BitAccess,
{
    type Item = D::Slot;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slot_idx < self.data.slots_count() {
            let v = self.data.get_slot(self.slot_idx);
            self.slot_idx += 1;
            Some(v)
        } else {
            None
        }
    }
}

/// An iterator over bits.
pub struct IterBits<'a, D, B> {
    slot_idx: usize,
    bit_idx: usize,
    data: &'a D,
    phantom: PhantomData<B>,
}

impl<D, B, N> Iterator for IterBits<'_, D, B>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
    N: Number,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slot_idx < self.data.slots_count() {
            let slot = self.data.get_slot(self.slot_idx);
            let v = B::get(slot, self.bit_idx);

            self.bit_idx = (self.bit_idx + 1) % N::BITS_COUNT;
            if self.bit_idx == 0 {
                self.slot_idx += 1;
            }

            Some(v)
        } else {
            None
        }
    }
}
