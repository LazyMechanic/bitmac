use std::marker::PhantomData;

use crate::{container::ContainerRead, BitAccess};

pub struct IntoIter<D, B> {
    idx: usize,
    data: D,
    phantom: PhantomData<B>,
}

impl<D, B> IntoIter<D, B> {
    pub fn new(data: D) -> Self {
        Self {
            idx: 0,
            data,
            phantom: Default::default(),
        }
    }
}

impl<D, B> Iterator for IntoIter<D, B>
where
    D: ContainerRead<B>,
    B: BitAccess,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.data.bits_count() {
            let v = self.data.get_bit(self.idx);
            self.idx += 1;
            Some(v)
        } else {
            None
        }
    }
}

pub struct Iter<'a, D, B> {
    idx: usize,
    data: &'a D,
    phantom: PhantomData<B>,
}

impl<'a, D, B> Iter<'a, D, B> {
    pub fn new(data: &'a D) -> Self {
        Self {
            idx: 0,
            data,
            phantom: Default::default(),
        }
    }
}

impl<D, B> Iterator for Iter<'_, D, B>
where
    D: ContainerRead<B>,
    B: BitAccess,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.data.bits_count() {
            let v = self.data.get_bit(self.idx);
            self.idx += 1;
            Some(v)
        } else {
            None
        }
    }
}
