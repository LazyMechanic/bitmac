use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use crate::{
    container::{ContainerRead, ContainerWrite},
    intersection::{try_intersection_impl, try_intersection_in_impl, Intersection, IntersectionIn},
    iter::{IntoIter, Iter},
    number::Number,
    union::{try_union_impl, try_union_in_impl, Union, UnionIn},
    with_slots::TryWithSlots,
    BitAccess, IntersectionError, OutOfBoundsError, UnionError, WithSlotsError,
};

#[derive(Default, Clone, Eq, PartialEq)]
pub struct StaticBitmap<D, B> {
    data: D,
    phantom: PhantomData<B>,
}

impl<D, B> StaticBitmap<D, B> {
    pub fn into_inner(self) -> D {
        self.data
    }
}

impl<D, N, B> StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    pub fn get(&self, idx: usize) -> bool {
        self.data.get_bit(idx)
    }
}

impl<D, N, B> StaticBitmap<D, B>
where
    D: ContainerWrite<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    pub fn set(&mut self, idx: usize, val: bool) {
        let _ = self.try_set(idx, val);
    }

    pub fn try_set(&mut self, idx: usize, val: bool) -> Result<(), OutOfBoundsError> {
        self.data.try_set_bit(idx, val)
    }
}

impl<D, B> AsRef<D> for StaticBitmap<D, B> {
    fn as_ref(&self) -> &D {
        &self.data
    }
}

impl<D, B> AsMut<D> for StaticBitmap<D, B> {
    fn as_mut(&mut self) -> &mut D {
        &mut self.data
    }
}

impl<D, B> ContainerRead<B> for StaticBitmap<D, B>
where
    D: ContainerRead<B>,
    B: BitAccess,
{
    type Slot = D::Slot;

    fn get_slot(&self, idx: usize) -> Self::Slot {
        self.data.get_slot(idx)
    }

    fn slots_count(&self) -> usize {
        self.data.slots_count()
    }
}

impl<D, B> ContainerWrite<B> for StaticBitmap<D, B>
where
    D: ContainerWrite<B>,
    B: BitAccess,
{
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        self.data.get_mut_slot(idx)
    }
}

impl<D, B> TryWithSlots for StaticBitmap<D, B>
where
    D: TryWithSlots,
    B: BitAccess,
{
    fn try_with_slots(len: usize) -> Result<Self, WithSlotsError> {
        Ok(Self {
            data: D::try_with_slots(len)?,
            phantom: Default::default(),
        })
    }
}

impl<D, N, B> Debug for StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for i in 0..self.data.slots_count() {
            let slot = self.data.get_slot(i);
            for j in 0..N::BYTES_COUNT {
                let byte = (slot >> (j * 8)) & N::BYTE_MASK;
                list.entry(&format_args!("{:#010b}", byte));
            }
        }
        list.finish()
    }
}

impl<D, B> From<D> for StaticBitmap<D, B> {
    fn from(f: D) -> Self {
        Self {
            data: f,
            phantom: Default::default(),
        }
    }
}

impl<D, N, B> IntoIterator for StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
{
    type Item = <IntoIter<D, B> as Iterator>::Item;
    type IntoIter = IntoIter<D, B>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.data)
    }
}

impl<'a, D, N, B> IntoIterator for &'a StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
{
    type Item = <Iter<'a, D, B> as Iterator>::Item;
    type IntoIter = Iter<'a, D, B>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(&self.data)
    }
}

impl<D, B, Rhs, N> IntersectionIn<Rhs, N, B> for StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
{
    fn intersection_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst)
    where
        Dst: ContainerWrite<B, Slot = N>,
    {
        try_intersection_in_impl(&self.data, rhs, dst).unwrap();
    }

    fn try_intersection_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst) -> Result<(), IntersectionError>
    where
        Dst: ContainerWrite<B, Slot = N>,
    {
        try_intersection_in_impl(&self.data, rhs, dst)
    }
}

impl<D, B, Rhs, N> Intersection<Rhs, N, B> for StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
{
    fn intersection<Dst>(&self, rhs: &Rhs) -> Dst
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots,
    {
        try_intersection_impl(&self.data, rhs).unwrap()
    }

    fn try_intersection<Dst>(&self, rhs: &Rhs) -> Result<Dst, IntersectionError>
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots,
    {
        try_intersection_impl(&self.data, rhs)
    }
}

impl<D, B, Rhs, N> UnionIn<Rhs, N, B> for StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
{
    fn union_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst)
    where
        Dst: ContainerWrite<B, Slot = N>,
    {
        try_union_in_impl(&self.data, rhs, dst).unwrap();
    }

    fn try_union_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst) -> Result<(), UnionError>
    where
        Dst: ContainerWrite<B, Slot = N>,
    {
        try_union_in_impl(&self.data, rhs, dst)
    }
}

impl<D, B, Rhs, N> Union<Rhs, N, B> for StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
{
    fn union<Dst>(&self, rhs: &Rhs) -> Dst
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots,
    {
        try_union_impl(&self.data, rhs).unwrap()
    }

    fn try_union<Dst>(&self, rhs: &Rhs) -> Result<Dst, UnionError>
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots,
    {
        try_union_impl(&self.data, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LSB;

    #[test]
    #[rustfmt::skip]
    fn get_bit() {
        // Number
        assert!(StaticBitmap::<u8, LSB>::from(1 << 0).get(0));
        assert!(StaticBitmap::<u8, LSB>::from(1 << 1).get(1));
        assert!(StaticBitmap::<u8, LSB>::from(1 << 2).get(2));
        assert!(StaticBitmap::<u8, LSB>::from(1 << 3).get(3));
        assert!(StaticBitmap::<u8, LSB>::from(1 << 4).get(4));
        assert!(StaticBitmap::<u8, LSB>::from(1 << 5).get(5));
        assert!(StaticBitmap::<u8, LSB>::from(1 << 6).get(6));
        assert!(StaticBitmap::<u8, LSB>::from(1 << 7).get(7));
        assert!(!StaticBitmap::<u8, LSB>::from(0b1111_1111).get(8));
        
        assert!(StaticBitmap::<u16, LSB>::from(1 << 0).get(0));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 1).get(1));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 2).get(2));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 3).get(3));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 4).get(4));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 5).get(5));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 6).get(6));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 7).get(7));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 8).get(8));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 9).get(9));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 10).get(10));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 11).get(11));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 12).get(12));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 13).get(13));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 14).get(14));
        assert!(StaticBitmap::<u16, LSB>::from(1 << 15).get(15));
        assert!(!StaticBitmap::<u16, LSB>::from(0b1111_1111_1111_1111).get(16));
        
        assert!(StaticBitmap::<u32, LSB>::from(1 << 0).get(0));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 1).get(1));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 2).get(2));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 3).get(3));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 4).get(4));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 5).get(5));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 6).get(6));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 7).get(7));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 8).get(8));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 9).get(9));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 10).get(10));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 11).get(11));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 12).get(12));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 13).get(13));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 14).get(14));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 15).get(15));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 16).get(16));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 17).get(17));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 18).get(18));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 19).get(19));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 20).get(20));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 21).get(21));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 22).get(22));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 23).get(23));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 24).get(24));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 25).get(25));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 26).get(26));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 27).get(27));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 28).get(28));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 29).get(29));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 30).get(30));
        assert!(StaticBitmap::<u32, LSB>::from(1 << 31).get(31));
        assert!(!StaticBitmap::<u32, LSB>::from(0b0000_0000_0000_0000_0000_0000_0000_0000).get(32));
        
        assert!(StaticBitmap::<u64, LSB>::from(1 << 0).get(0));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 1).get(1));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 2).get(2));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 3).get(3));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 4).get(4));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 5).get(5));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 6).get(6));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 7).get(7));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 8).get(8));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 9).get(9));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 10).get(10));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 11).get(11));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 12).get(12));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 13).get(13));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 14).get(14));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 15).get(15));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 16).get(16));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 17).get(17));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 18).get(18));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 19).get(19));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 20).get(20));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 21).get(21));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 22).get(22));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 23).get(23));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 24).get(24));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 25).get(25));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 26).get(26));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 27).get(27));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 28).get(28));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 29).get(29));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 30).get(30));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 31).get(31));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 32).get(32));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 33).get(33));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 34).get(34));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 35).get(35));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 36).get(36));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 37).get(37));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 38).get(38));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 39).get(39));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 40).get(40));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 41).get(41));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 42).get(42));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 43).get(43));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 44).get(44));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 45).get(45));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 46).get(46));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 47).get(47));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 48).get(48));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 49).get(49));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 50).get(50));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 51).get(51));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 52).get(52));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 53).get(53));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 54).get(54));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 55).get(55));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 56).get(56));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 57).get(57));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 58).get(58));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 59).get(59));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 60).get(60));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 61).get(61));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 62).get(62));
        assert!(StaticBitmap::<u64, LSB>::from(1 << 63).get(63));
        assert!(!StaticBitmap::<u64, LSB>::from(0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111).get(64));
        
        // Slice
        assert!(StaticBitmap::<&'static [u8], LSB>::from(&[1u8][..]).get(0));
        assert!(StaticBitmap::<&'static [u8], LSB>::from(&[1u8, 1][..]).get(8));
        assert!(!StaticBitmap::<&'static [u8], LSB>::from(&[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..]).get(999));
        assert!(StaticBitmap::<&'static [u16], LSB>::from(&[1u16][..]).get(0));
        assert!(StaticBitmap::<&'static [u16], LSB>::from(&[1u16, 1u16][..]).get(16));
        assert!(!StaticBitmap::<&'static [u16], LSB>::from(&[0b1111_1111_1111_1111u16, 0b1111_1111_1111_1111, 0b1111_1111_1111_1111][..]).get(999));
        assert!(StaticBitmap::<&'static [u32], LSB>::from(&[1u32][..]).get(0));
        assert!(StaticBitmap::<&'static [u32], LSB>::from(&[1u32, 1][..]).get(32));
        assert!(!StaticBitmap::<&'static [u32], LSB>::from(&[0b1111_1111_1111_1111_1111_1111_1111_1111u32, 0b1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111][..]).get(999));
        assert!(StaticBitmap::<&'static [u64], LSB>::from(&[1u64][..]).get(0));
        assert!(StaticBitmap::<&'static [u64], LSB>::from(&[1u64, 1][..]).get(64));
        assert!(!StaticBitmap::<&'static [u64], LSB>::from(&[0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111u64, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111][..]).get(999));

        let v = &[1u8][..];
        assert!(StaticBitmap::<&[u8], LSB>::from(v).get(0));
        let v = &[1u8, 1][..];
        assert!(StaticBitmap::<&[u8], LSB>::from(v).get(8));
        let v = &[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..];
        assert!(!StaticBitmap::<&[u8], LSB>::from(v).get(999));
        let v = &[1u16][..];
        assert!(StaticBitmap::<&[u16], LSB>::from(v).get(0));
        let v = &[1u16, 1u16][..];
        assert!(StaticBitmap::<&[u16], LSB>::from(v).get(16));
        let v = &[0b1111_1111_1111_1111u16, 0b1111_1111_1111_1111, 0b1111_1111_1111_1111][..];
        assert!(!StaticBitmap::<&[u16], LSB>::from(v).get(999));
        let v = &[1u32][..];
        assert!(StaticBitmap::<&[u32], LSB>::from(v).get(0));
        let v = &[1u32, 1][..];
        assert!(StaticBitmap::<&[u32], LSB>::from(v).get(32));
        let v = &[0b1111_1111_1111_1111_1111_1111_1111_1111u32, 0b1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111][..];
        assert!(!StaticBitmap::<&[u32], LSB>::from(v).get(999));
        let v = &[1u64][..];
        assert!(StaticBitmap::<&[u64], LSB>::from(v).get(0));
        let v = &[1u64, 1][..];
        assert!(StaticBitmap::<&[u64], LSB>::from(v).get(64));
        let v = &[0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111u64, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111][..];
        assert!(!StaticBitmap::<&[u64], LSB>::from(v).get(999));

        // Array
        assert!(StaticBitmap::<[u8; 1], LSB>::from([1; 1]).get(0));
        assert!(StaticBitmap::<[u8; 2], LSB>::from([1; 2]).get(8));
        assert!(!StaticBitmap::<[u8; 3], LSB>::from([0b1111_1111; 3]).get(999));
        assert!(StaticBitmap::<[u16; 1], LSB>::from([1; 1]).get(0));
        assert!(StaticBitmap::<[u16; 2], LSB>::from([1; 2]).get(16));
        assert!(!StaticBitmap::<[u16; 3], LSB>::from([0b1111_1111_1111_1111; 3]).get(999));
        assert!(StaticBitmap::<[u32; 1], LSB>::from([1; 1]).get(0));
        assert!(StaticBitmap::<[u32; 2], LSB>::from([1; 2]).get(32));
        assert!(!StaticBitmap::<[u32; 3], LSB>::from([0b1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));
        assert!(StaticBitmap::<[u64; 1], LSB>::from([1; 1]).get(0));
        assert!(StaticBitmap::<[u64; 2], LSB>::from([1; 2]).get(64));
        assert!(!StaticBitmap::<[u64; 3], LSB>::from([0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));

        // Vec
        assert!(StaticBitmap::<Vec<u8>, LSB>::from(vec![1; 1]).get(0));
        assert!(StaticBitmap::<Vec<u8>, LSB>::from(vec![1; 2]).get(8));
        assert!(!StaticBitmap::<Vec<u8>, LSB>::from(vec![0b1111_1111; 3]).get(999));
        assert!(StaticBitmap::<Vec<u16>, LSB>::from(vec![1; 1]).get(0));
        assert!(StaticBitmap::<Vec<u16>, LSB>::from(vec![1; 2]).get(16));
        assert!(!StaticBitmap::<Vec<u16>, LSB>::from(vec![0b1111_1111_1111_1111; 3]).get(999));
        assert!(StaticBitmap::<Vec<u32>, LSB>::from(vec![1; 1]).get(0));
        assert!(StaticBitmap::<Vec<u32>, LSB>::from(vec![1; 2]).get(32));
        assert!(!StaticBitmap::<Vec<u32>, LSB>::from(vec![0b1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));
        assert!(StaticBitmap::<Vec<u64>, LSB>::from(vec![1; 1]).get(0));
        assert!(StaticBitmap::<Vec<u64>, LSB>::from(vec![1; 2]).get(64));
        assert!(!StaticBitmap::<Vec<u64>, LSB>::from(vec![0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));

        // Bytes
        #[cfg(feature = "bytes")]
        {
            use bytes::{Bytes, BytesMut};
            assert!(StaticBitmap::<Bytes, LSB>::from(Bytes::from_static(&[1])).get(0));
            assert!(StaticBitmap::<Bytes, LSB>::from(Bytes::from_static(&[1, 1])).get(8));
            assert!(!StaticBitmap::<Bytes, LSB>::from(Bytes::from_static(&[0b1111_1111, 0b1111_1111, 0b1111_1111])).get(999));
            assert!(StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[1u8][..])).get(0));
            assert!(StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[1u8, 1][..])).get(8));
            assert!(!StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..])).get(999));
        }

        // SmallVec
        #[cfg(feature = "smallvec")]
        {
            use smallvec::SmallVec;
            assert!(StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(SmallVec::from([1u8])).get(0));
            assert!(StaticBitmap::<SmallVec<[u8; 2]>, LSB>::from(SmallVec::from([1u8, 1])).get(8));
            assert!(!StaticBitmap::<SmallVec<[u8; 3]>, LSB>::from(SmallVec::from([0b1111_1111u8, 0b1111_1111, 0b1111_1111])).get(999));
        }
    }

    #[test]
    #[rustfmt::skip]
    fn set_bit() {
        // Number
        let mut v = StaticBitmap::<u8, LSB>::default();
        v.set(0, true);
        v.set(7, true);
        assert!(v.try_set(8, true).is_err());
        assert!(v.get(0));
        assert!(v.get(7));

        let mut v = StaticBitmap::<u16, LSB>::default();
        v.set(0, true);
        v.set(15, true);
        assert!(v.try_set(16, true).is_err());
        assert!(v.get(0));
        assert!(v.get(15));

        let mut v = StaticBitmap::<u32, LSB>::default();
        v.set(0, true);
        v.set(31, true);
        assert!(v.try_set(32, true).is_err());
        assert!(v.get(0));
        assert!(v.get(31));
        
        let mut v = StaticBitmap::<u64, LSB>::default();
        v.set(0, true);
        v.set(63, true);
        assert!(v.try_set(64, true).is_err());
        assert!(v.get(0));
        assert!(v.get(63));
        
        // Slice
        let mut inner = vec![0, 0];
        let mut v = StaticBitmap::<&mut [u8], LSB>::from(inner.as_mut_slice());
        v.set(0, true);
        v.set(15, true);
        assert!(v.try_set(16, true).is_err());
        assert!(v.get(0));
        assert!(v.get(15));

        let mut inner = vec![0, 0];
        let mut v = StaticBitmap::<&mut [u16], LSB>::from(inner.as_mut_slice());
        v.set(0, true);
        v.set(31, true);
        assert!(v.try_set(32, true).is_err());
        assert!(v.get(0));
        assert!(v.get(31));

        let mut inner = vec![0, 0];
        let mut v = StaticBitmap::<&mut [u32], LSB>::from(inner.as_mut_slice());
        v.set(0, true);
        v.set(63, true);
        assert!(v.try_set(64, true).is_err());
        assert!(v.get(0));
        assert!(v.get(63));

        let mut inner = vec![0, 0];
        let mut v = StaticBitmap::<&mut [u64], LSB>::from(inner.as_mut_slice());
        v.set(0, true);
        v.set(127, true);
        assert!(v.try_set(128, true).is_err());
        assert!(v.get(0));
        assert!(v.get(127));

        // Array
        let mut v = StaticBitmap::<[u8; 2], LSB>::default();
        v.set(0, true);
        v.set(15, true);
        assert!(v.try_set(16, true).is_err());
        assert!(v.get(0));
        assert!(v.get(15));

        let mut v = StaticBitmap::<[u16; 2], LSB>::default();
        v.set(0, true);
        v.set(31, true);
        assert!(v.try_set(32, true).is_err());
        assert!(v.get(0));
        assert!(v.get(31));

        let mut v = StaticBitmap::<[u32; 2], LSB>::default();
        v.set(0, true);
        v.set(63, true);
        assert!(v.try_set(64, true).is_err());
        assert!(v.get(0));
        assert!(v.get(63));

        let mut v = StaticBitmap::<[u64; 2], LSB>::default();
        v.set(0, true);
        v.set(127, true);
        assert!(v.try_set(128, true).is_err());
        assert!(v.get(0));
        assert!(v.get(127));
        
        // Vec
        let mut v = StaticBitmap::<Vec<u8>, LSB>::from(vec![0, 0]);
        v.set(0, true);
        v.set(15, true);
        assert!(v.try_set(16, true).is_err());
        assert!(v.get(0));
        assert!(v.get(15));

        let mut v = StaticBitmap::<Vec<u16>, LSB>::from(vec![0, 0]);
        v.set(0, true);
        v.set(31, true);
        assert!(v.try_set(32, true).is_err());
        assert!(v.get(0));
        assert!(v.get(31));

        let mut v = StaticBitmap::<Vec<u32>, LSB>::from(vec![0, 0]);
        v.set(0, true);
        v.set(63, true);
        assert!(v.try_set(64, true).is_err());
        assert!(v.get(0));
        assert!(v.get(63));

        let mut v = StaticBitmap::<Vec<u64>, LSB>::from(vec![0, 0]);
        v.set(0, true);
        v.set(127, true);
        assert!(v.try_set(128, true).is_err());
        assert!(v.get(0));
        assert!(v.get(127));

        // Bytes
        #[cfg(feature = "bytes")]
        {
            use bytes::{BytesMut};
            let mut v = StaticBitmap::<BytesMut, LSB>::from(BytesMut::zeroed(2));
            v.set(0, true);
            v.set(15, true);
            assert!(v.try_set(16, true).is_err());
            assert!(v.get(0));
            assert!(v.get(15));
        }
        
        #[cfg(feature = "smallvec")]
        {
            use smallvec::{SmallVec, smallvec};
            let mut v = StaticBitmap::<SmallVec<[u8; 2]>, LSB>::from(smallvec![0, 0]);
            v.set(0, true);
            v.set(15, true);
            assert!(v.try_set(16, true).is_err());
            assert!(v.get(0));
            assert!(v.get(15));
        }
    }

    #[test]
    fn intersection() {
        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let exp = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        assert_eq!(lhs.intersection::<StaticBitmap::<u8, LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_0100]);
        assert_eq!(lhs.intersection::<StaticBitmap::<[u8; 1], LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_0100]);
        assert_eq!(lhs.intersection::<StaticBitmap::<Vec<u8>, LSB>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0010_0100][..]));
            assert_eq!(lhs.intersection::<StaticBitmap::<BytesMut, LSB>>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
            let exp = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0010_0100]);
            assert_eq!(
                lhs.intersection::<StaticBitmap::<SmallVec<[u8; 1]>, LSB>>(&rhs),
                exp
            );
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let exp = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        assert_eq!(lhs.intersection::<StaticBitmap::<u8, LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_0100]);
        assert_eq!(lhs.intersection::<StaticBitmap::<[u8; 1], LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_0100]);
        assert_eq!(lhs.intersection::<StaticBitmap::<Vec<u8>, LSB>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0010_0100][..]));
            assert_eq!(lhs.intersection::<StaticBitmap::<BytesMut, LSB>>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
            let exp = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0010_0100]);
            assert_eq!(
                lhs.intersection::<StaticBitmap::<SmallVec<[u8; 1]>, LSB>>(&rhs),
                exp
            );
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_0100]);
        assert_eq!(lhs.intersection::<StaticBitmap::<[u8; 1], LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_0100]);
        assert_eq!(lhs.intersection::<StaticBitmap::<[u8; 1], LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_0100]);
        assert_eq!(lhs.intersection::<StaticBitmap::<Vec<u8>, LSB>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0010_0100][..]));
            assert_eq!(lhs.intersection::<StaticBitmap::<BytesMut, LSB>>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
            let exp = StaticBitmap::<SmallVec<[u8; 2]>, LSB>::from(smallvec![0b0010_0100]);
            assert_eq!(
                lhs.intersection::<StaticBitmap::<SmallVec<[u8; 2]>, LSB>>(&rhs),
                exp
            );
        }

        ////////////////////////////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let exp: u8 = 0b0010_0100;
        assert_eq!(lhs.intersection::<u8>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let exp: [u8; 1] = [0b0010_0100];
        assert_eq!(lhs.intersection::<[u8; 1]>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let exp: Vec<u8> = vec![0b0010_0100];
        assert_eq!(lhs.intersection::<Vec<u8>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: u8 = 0b0010_0100;
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            assert_eq!(lhs.intersection::<BytesMut>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: u8 = 0b0010_0100;
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_0100];
            assert_eq!(lhs.intersection::<SmallVec<[u8; 1]>>(&rhs), exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: u8 = 0b0010_0100;
        assert_eq!(lhs.intersection::<u8>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        assert_eq!(lhs.intersection::<[u8; 1]>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        assert_eq!(lhs.intersection::<Vec<u8>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            assert_eq!(lhs.intersection::<BytesMut>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_0100];
            assert_eq!(lhs.intersection::<SmallVec<[u8; 1]>>(&rhs), exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        assert_eq!(lhs.intersection::<[u8; 1]>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        assert_eq!(lhs.intersection::<[u8; 1]>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        assert_eq!(lhs.intersection::<Vec<u8>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            assert_eq!(lhs.intersection::<BytesMut>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
            let exp: SmallVec<[u8; 2]> = smallvec![0b0010_0100];
            assert_eq!(lhs.intersection::<SmallVec<[u8; 2]>>(&rhs), exp);
        }
    }

    #[test]
    fn try_intersection() {
        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        assert!(lhs
            .try_intersection::<StaticBitmap::<[u8; 10], LSB>>(&rhs)
            .is_err());

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        assert!(lhs
            .try_intersection::<StaticBitmap::<[u8; 3], LSB>>(&rhs)
            .is_err());

        //////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        assert!(lhs.try_intersection::<[u8; 10]>(&rhs).is_err());

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        assert!(lhs.try_intersection::<[u8; 3]>(&rhs).is_err());
    }

    #[test]
    fn intersection_in() {
        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let mut dst = StaticBitmap::<u8, LSB>::from(0b0000_0000);
        let exp = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let mut dst = StaticBitmap::<[u8; 1], LSB>::from([0b0000_0000]);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_0100]);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let v = &mut [0b0000_0000][..];
        let mut dst = StaticBitmap::<&mut [u8], LSB>::from(v);
        let v = &mut [0b0010_0100][..];
        let exp = StaticBitmap::<&mut [u8], LSB>::from(v);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let mut dst = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0000_0000]);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_0100]);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
            let mut dst = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0000_0000][..]));
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0010_0100][..]));
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
            let mut dst = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0000_0000]);
            let exp = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0010_0100]);
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let mut dst = StaticBitmap::<u8, LSB>::from(0b0000_0000);
        let exp = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let mut dst = StaticBitmap::<[u8; 1], LSB>::from([0b0000_0000]);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_0100]);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let mut dst = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0000_0000]);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_0100]);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
            let mut dst = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0000_0000][..]));
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0010_0100][..]));
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
            let mut dst = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0000_0000]);
            let exp = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0010_0100]);
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
        let mut dst = StaticBitmap::<[u8; 1], LSB>::from([0b0000_0000]);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_0100]);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
        let mut dst = StaticBitmap::<[u8; 1], LSB>::from([0b0000_0000]);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_0100]);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
        let mut dst = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0000_0000]);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_0100]);
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
            let mut dst = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0000_0000][..]));
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0010_0100][..]));
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0110_0000, 0b0000_0000]);
            let mut dst = StaticBitmap::<SmallVec<[u8; 2]>, LSB>::from(smallvec![0b0000_0000]);
            let exp = StaticBitmap::<SmallVec<[u8; 2]>, LSB>::from(smallvec![0b0010_0100]);
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        /////////////////////////////////////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let mut dst: u8 = 0b0000_0000;
        let exp: u8 = 0b0010_0100;
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let v = &mut [0b0000_0000][..];
        let mut dst: &mut [u8] = v;
        let v = &mut [0b0010_0100][..];
        let exp: &mut [u8] = v;
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let mut dst: Vec<u8> = vec![0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: u8 = 0b0010_0100;
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: u8 = 0b0010_0100;
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_0100];
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: u8 = 0b0000_0000;
        let exp: u8 = 0b0010_0100;
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_0100];
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        lhs.intersection_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
            let mut dst: SmallVec<[u8; 2]> = smallvec![0b0000_0000];
            let exp: SmallVec<[u8; 2]> = smallvec![0b0010_0100];
            lhs.intersection_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }
    }

    #[test]
    fn try_intersection_in() {
        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
        let mut dst = StaticBitmap::<[u8; 1], LSB>::from([0b0000_0000; 1]);
        assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
        let mut dst = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0000_0000; 1]);
        assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
        let v = &mut [0b0000_0000][..];
        let mut dst = StaticBitmap::<&mut [u8], LSB>::from(v);
        assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
            let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
            let mut dst = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0000_0000][..]));
            assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
            let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
            let mut dst = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0000_0000]);
            assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());
        }

        //////////////////

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000; 1];
        assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000; 1];
        assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let v = &mut [0b0000_0000][..];
        let mut dst: &mut [u8] = v;
        assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            assert!(lhs.try_intersection_in(&rhs, &mut dst).is_err());
        }
    }

    #[test]
    fn union() {
        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let exp = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        assert_eq!(lhs.union::<StaticBitmap<u8, LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_1100]);
        assert_eq!(lhs.union::<StaticBitmap<[u8; 1], LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_1100]);
        assert_eq!(lhs.union::<StaticBitmap<Vec<u8>, LSB>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0010_1100][..]));
            assert_eq!(lhs.union::<StaticBitmap<BytesMut, LSB>>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
            let exp = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0010_1100]);
            assert_eq!(lhs.union::<StaticBitmap<SmallVec<[u8; 1]>, LSB>>(&rhs), exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let exp = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        assert_eq!(lhs.union::<StaticBitmap<[u8; 2], LSB>>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_1100, 0b0000_0000]);
        assert_eq!(lhs.union::<StaticBitmap<Vec<u8>, LSB>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(
                &[0b0010_1100, 0b0000_0000][..],
            ));
            assert_eq!(lhs.union::<StaticBitmap<BytesMut, LSB>>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
            let exp =
                StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0010_1100, 0b0000_0000]);
            assert_eq!(lhs.union::<StaticBitmap<SmallVec<[u8; 1]>, LSB>>(&rhs), exp);
        }

        ////////////////////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let exp: u8 = 0b0010_1100;
        assert_eq!(lhs.union::<u8>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let exp: [u8; 1] = [0b0010_1100];
        assert_eq!(lhs.union::<[u8; 1]>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let exp: Vec<u8> = vec![0b0010_1100];
        assert_eq!(lhs.union::<Vec<u8>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: u8 = 0b0010_0100;
            let exp: BytesMut = BytesMut::from(&[0b0010_1100][..]);
            assert_eq!(lhs.union::<BytesMut>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: u8 = 0b0010_0100;
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_1100];
            assert_eq!(lhs.union::<SmallVec<[u8; 1]>>(&rhs), exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: [u8; 2] = [0b0010_1100, 0b0000_0000];
        assert_eq!(lhs.union::<[u8; 2]>(&rhs), exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_1100, 0b0000_0000];
        assert_eq!(lhs.union::<Vec<u8>>(&rhs), exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let exp: BytesMut = BytesMut::from(&[0b0010_1100, 0b0000_0000][..]);
            assert_eq!(lhs.union::<BytesMut>(&rhs), exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_1100, 0b0000_0000];
            assert_eq!(lhs.union::<SmallVec<[u8; 1]>>(&rhs), exp);
        }
    }

    #[test]
    fn try_union() {
        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        assert!(lhs.try_union::<StaticBitmap<[u8; 10], LSB>>(&rhs).is_err());

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        assert!(lhs.try_union::<StaticBitmap<[u8; 3], LSB>>(&rhs).is_err());

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        assert!(lhs.try_union::<StaticBitmap<u8, LSB>>(&rhs).is_err());

        //////////////////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        assert!(lhs.try_union::<[u8; 10]>(&rhs).is_err());

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        assert!(lhs.try_union::<[u8; 3]>(&rhs).is_err());

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        assert!(lhs.try_union::<u8>(&rhs).is_err());
    }

    #[test]
    fn union_in() {
        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let mut dst = StaticBitmap::<u8, LSB>::from(0b0000_0000);
        let exp = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let mut dst = StaticBitmap::<[u8; 1], LSB>::from([0b0000_0000]);
        let exp = StaticBitmap::<[u8; 1], LSB>::from([0b0010_1100]);
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
        let mut dst = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0000_0000]);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_1100]);
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
            let mut dst = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0000_0000][..]));
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(&[0b0010_1100][..]));
            lhs.union_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<u8, LSB>::from(0b0010_0100);
            let mut dst = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0000_0000]);
            let exp = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0010_1100]);
            lhs.union_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let mut dst = StaticBitmap::<[u8; 2], LSB>::from([0b0000_0000, 0b0000_0000]);
        let exp = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
        let mut dst = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0000_0000, 0b0000_0000]);
        let exp = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0010_1100, 0b0000_0000]);
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
            let mut dst = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(
                &[0b0000_0000, 0b0000_0000][..],
            ));
            let exp = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(
                &[0b0010_1100, 0b0000_0000][..],
            ));
            lhs.union_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_0100, 0b0000_0000]);
            let mut dst =
                StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0000_0000, 0b0000_0000]);
            let exp =
                StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0010_1100, 0b0000_0000]);
            lhs.union_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        /////////////////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let mut dst: u8 = 0b0000_0000;
        let exp: u8 = 0b0010_1100;
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_1100];
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: u8 = 0b0010_0100;
        let mut dst: Vec<u8> = vec![0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_1100];
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: u8 = 0b0010_0100;
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_1100][..]);
            lhs.union_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: u8 = 0b0010_0100;
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_1100];
            lhs.union_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        /////////

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: [u8; 2] = [0b0000_0000, 0b0000_0000];
        let exp: [u8; 2] = [0b0010_1100, 0b0000_0000];
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000, 0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_1100, 0b0000_0000];
        lhs.union_in(&rhs, &mut dst);
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000, 0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_1100, 0b0000_0000][..]);
            lhs.union_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<u8, LSB>::from(0b0010_1100);
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000, 0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_1100, 0b0000_0000];
            lhs.union_in(&rhs, &mut dst);
            assert_eq!(dst, exp);
        }
    }

    #[test]
    fn try_union_in() {
        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
        let mut dst = StaticBitmap::<[u8; 2], LSB>::from([0b0000_0000; 2]);
        assert!(lhs.try_union_in(&rhs, &mut dst).is_err());

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
        let mut dst = StaticBitmap::<Vec<u8>, LSB>::from(vec![0b0000_0000; 2]);
        assert!(lhs.try_union_in(&rhs, &mut dst).is_err());

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
        let v = &mut [0b0000_0000, 0b0000_0000][..];
        let mut dst = StaticBitmap::<&mut [u8], LSB>::from(v);
        assert!(lhs.try_union_in(&rhs, &mut dst).is_err());

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
            let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
            let mut dst = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(
                &[0b0000_0000, 0b0000_0000][..],
            ));
            assert!(lhs.try_union_in(&rhs, &mut dst).is_err());
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
            let rhs = StaticBitmap::<[u8; 3], LSB>::from([0b0010_0100, 0b0000_0000, 0b0000_0000]);
            let mut dst = StaticBitmap::<SmallVec<[u8; 1]>, LSB>::from(smallvec![0b0000_0000; 2]);
            assert!(lhs.try_union_in(&rhs, &mut dst).is_err());
        }

        //////////////////////////

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: [u8; 2] = [0b0000_0000; 2];
        assert!(lhs.try_union_in(&rhs, &mut dst).is_err());

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000; 2];
        assert!(lhs.try_union_in(&rhs, &mut dst).is_err());

        let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let v = &mut [0b0000_0000, 0b0000_0000][..];
        let mut dst: &mut [u8] = v;
        assert!(lhs.try_union_in(&rhs, &mut dst).is_err());

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst = StaticBitmap::<BytesMut, LSB>::from(BytesMut::from(
                &[0b0000_0000, 0b0000_0000][..],
            ));
            assert!(lhs.try_union_in(&rhs, &mut dst).is_err());
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs = StaticBitmap::<[u8; 2], LSB>::from([0b0010_1100, 0b0000_0000]);
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000; 2];
            assert!(lhs.try_union_in(&rhs, &mut dst).is_err());
        }
    }
}
