use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use crate::{
    container::{ContainerRead, ContainerWrite},
    intersection::{
        intersection_len_impl, try_intersection_impl, try_intersection_in_impl, Intersection,
    },
    iter::{IntoIter, Iter},
    number::Number,
    union::{try_union_impl, try_union_in_impl, union_len_impl, Union},
    with_slots::TryWithSlots,
    BitAccess, IntersectionError, OutOfBoundsError, UnionError, WithSlotsError,
};

/// A bitmap that cannot be resized.
///
/// Any structure that implements the [`ContainerRead`] (for read-only access) and [`ContainerWrite`]
/// (for mutable access) traits can be a container of bitmap (e.g. `[T; N]`, `&[T]`, `Vec<T>`, etc.).
///
/// Usage example:
/// ```
/// # fn main() {
/// use bitmac::{StaticBitmap, LSB};
///
/// // You can directly check every single bit
/// let bitmap = StaticBitmap::<_, LSB>::new([0b0000_0001u8, 0b0000_1000]);
/// assert!(bitmap.get(0));
/// assert!(bitmap.get(11));
/// assert!(!bitmap.get(13));
/// // Out of bounds bits always return false
/// assert!(!bitmap.get(128));
///
/// // You can iterate over bits
/// let bitmap = StaticBitmap::<_, LSB>::new([0b0000_1001u8, 0b0000_1000]);
/// let mut iter = bitmap.iter().by_bits().enumerate();
/// assert_eq!(iter.next(), Some((0, true)));
/// assert_eq!(iter.next(), Some((1, false)));
/// assert_eq!(iter.next(), Some((2, false)));
/// assert_eq!(iter.next(), Some((3, true)));
/// assert_eq!(iter.next(), Some((4, false)));
///
/// // You can check multiple bits at the same time through the intersection
/// use bitmac::Intersection;
/// let bitmap = StaticBitmap::<_, LSB>::new([0b0000_1001u8, 0b0000_1000]);
/// // .. by creating specific new container for result
/// let test = [0b0000_1001u8, 0b0000_0000];
/// assert_eq!(bitmap.intersection::<[u8; 2]>(&test), test);
/// // .. by using preallocated container for result
/// let test = [0b0000_1001u8, 0b0000_0000];
/// let mut result = [0u8; 2];
/// bitmap.intersection_in(&test, &mut result);
/// assert_eq!(result, test);
/// // .. by comparing length of difference that is equivalent to count of ones (bits) in result
/// let test = [0b0000_1001u8, 0b0000_0000];
/// assert_eq!(bitmap.intersection_len(&test), test.iter().fold(0, |acc, &v| acc + v.count_ones() as usize));
///
/// // You can directly change every single bit
/// let mut bitmap = StaticBitmap::<_, LSB>::new([0b0000_1001u8, 0b0001_1000]);
/// assert!(bitmap.get(0));
/// assert!(bitmap.get(3));
/// assert!(bitmap.get(11));
/// assert!(bitmap.get(12));
/// assert!(!bitmap.get(13));
/// assert!(!bitmap.get(128));
/// bitmap.set(12, false);
/// assert!(!bitmap.get(12));
/// bitmap.set(13, true);
/// assert!(bitmap.get(13));
/// // Out of bounds bits return error
/// assert!(bitmap.try_set(128, true).is_err());
/// assert!(!bitmap.get(128));
/// # }
/// ```
#[derive(Default, Clone, Eq, PartialEq)]
pub struct StaticBitmap<D, B> {
    data: D,
    phantom: PhantomData<B>,
}

impl<D, N, B> StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    /// Creates new bitmap from container.
    pub fn new(data: D) -> Self {
        Self {
            data,
            phantom: Default::default(),
        }
    }

    /// Returns number of ones in the bitmap.
    pub fn count_ones(&self) -> usize {
        let mut res = 0;
        for v in self.iter() {
            res += v.count_ones() as usize;
        }
        res
    }

    /// Returns number of zeros in the bitmap.
    pub fn count_zeros(&self) -> usize {
        let mut res = 0;
        for v in self.iter() {
            res += v.count_zeros() as usize;
        }
        res
    }
}

impl<D, B> StaticBitmap<D, B> {
    /// Converts bitmap into inner container.
    pub fn into_inner(self) -> D {
        self.data
    }
}

impl<D, B> StaticBitmap<D, B>
where
    D: ContainerRead<B>,
    B: BitAccess,
{
    /// Gets single bit state.
    ///
    /// Usage example:
    /// ```
    /// use bitmac::{StaticBitmap, LSB};
    ///
    /// let bitmap = StaticBitmap::<_, LSB>::new([0b0000_0001u8, 0b0000_1000]);
    /// assert!(bitmap.get(0));
    /// assert!(bitmap.get(11));
    /// assert!(!bitmap.get(13));
    /// // Out of bounds bits always return false
    /// assert!(!bitmap.get(128));
    /// ```
    pub fn get(&self, idx: usize) -> bool {
        self.data.get_bit(idx)
    }

    /// Returns iterator over slots.
    pub fn iter(&self) -> Iter<'_, D, B> {
        Iter::new(&self.data)
    }
}

impl<D, B> StaticBitmap<D, B>
where
    D: ContainerWrite<B>,
    B: BitAccess,
{
    /// Sets new state for a single bit.
    ///
    /// ## Panic
    ///
    /// Panics if `idx` is out of bounds.
    /// See non-panic function [`try_set`].
    ///
    /// ## Usage example:
    /// ```
    /// use bitmac::{StaticBitmap, LSB};
    ///
    /// let mut bitmap = StaticBitmap::<_, LSB>::new([0b0000_1001u8, 0b0001_1000]);
    /// bitmap.set(12, false);
    /// assert!(!bitmap.get(12));
    /// bitmap.set(13, true);
    /// assert!(bitmap.get(13));
    /// ```
    ///
    /// [`try_set`]: crate::static_bitmap::StaticBitmap::try_set
    pub fn set(&mut self, idx: usize, val: bool) {
        self.try_set(idx, val).unwrap();
    }

    /// Sets new state for a single bit.
    ///
    /// Returns `Err(_)` if `idx` is out of bounds.
    ///
    /// ## Usage example:
    /// ```
    /// use bitmac::{StaticBitmap, LSB};
    ///
    /// let mut bitmap = StaticBitmap::<_, LSB>::new([0b0000_1001u8, 0b0001_1000]);
    /// assert!(bitmap.try_set(12, true).is_ok());
    /// assert!(bitmap.get(12));
    /// assert!(bitmap.try_set(12, false).is_ok());
    /// assert!(!bitmap.get(12));
    /// // Out of bounds bits return error
    /// assert!(bitmap.try_set(128, true).is_err());
    /// assert!(!bitmap.get(128));
    /// ```
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

impl<D, N, B> From<D> for StaticBitmap<D, B>
where
    D: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    fn from(f: D) -> Self {
        Self {
            data: f,
            phantom: Default::default(),
        }
    }
}

impl<D, B> IntoIterator for StaticBitmap<D, B>
where
    D: ContainerRead<B>,
    B: BitAccess,
{
    type Item = <IntoIter<D, B> as Iterator>::Item;
    type IntoIter = IntoIter<D, B>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.data)
    }
}

impl<'a, D, B> IntoIterator for &'a StaticBitmap<D, B>
where
    D: ContainerRead<B>,
    B: BitAccess,
{
    type Item = <Iter<'a, D, B> as Iterator>::Item;
    type IntoIter = Iter<'a, D, B>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, B, Rhs, N> Intersection<Rhs, N, B> for StaticBitmap<D, B>
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

    fn intersection_len(&self, rhs: &Rhs) -> usize {
        intersection_len_impl(&self.data, rhs)
    }
}

impl<D, B, Rhs, N> Union<Rhs, N, B> for StaticBitmap<D, B>
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

    fn union_len(&self, rhs: &Rhs) -> usize {
        union_len_impl(&self.data, rhs)
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
        assert!(StaticBitmap::<u8, LSB>::new(1 << 0).get(0));
        assert!(StaticBitmap::<u8, LSB>::new(1 << 1).get(1));
        assert!(StaticBitmap::<u8, LSB>::new(1 << 2).get(2));
        assert!(StaticBitmap::<u8, LSB>::new(1 << 3).get(3));
        assert!(StaticBitmap::<u8, LSB>::new(1 << 4).get(4));
        assert!(StaticBitmap::<u8, LSB>::new(1 << 5).get(5));
        assert!(StaticBitmap::<u8, LSB>::new(1 << 6).get(6));
        assert!(StaticBitmap::<u8, LSB>::new(1 << 7).get(7));
        assert!(!StaticBitmap::<u8, LSB>::new(0b1111_1111).get(8));
        
        assert!(StaticBitmap::<u16, LSB>::new(1 << 0).get(0));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 1).get(1));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 2).get(2));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 3).get(3));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 4).get(4));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 5).get(5));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 6).get(6));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 7).get(7));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 8).get(8));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 9).get(9));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 10).get(10));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 11).get(11));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 12).get(12));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 13).get(13));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 14).get(14));
        assert!(StaticBitmap::<u16, LSB>::new(1 << 15).get(15));
        assert!(!StaticBitmap::<u16, LSB>::new(0b1111_1111_1111_1111).get(16));
        
        assert!(StaticBitmap::<u32, LSB>::new(1 << 0).get(0));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 1).get(1));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 2).get(2));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 3).get(3));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 4).get(4));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 5).get(5));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 6).get(6));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 7).get(7));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 8).get(8));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 9).get(9));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 10).get(10));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 11).get(11));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 12).get(12));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 13).get(13));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 14).get(14));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 15).get(15));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 16).get(16));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 17).get(17));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 18).get(18));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 19).get(19));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 20).get(20));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 21).get(21));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 22).get(22));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 23).get(23));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 24).get(24));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 25).get(25));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 26).get(26));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 27).get(27));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 28).get(28));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 29).get(29));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 30).get(30));
        assert!(StaticBitmap::<u32, LSB>::new(1 << 31).get(31));
        assert!(!StaticBitmap::<u32, LSB>::new(0b0000_0000_0000_0000_0000_0000_0000_0000).get(32));
        
        assert!(StaticBitmap::<u64, LSB>::new(1 << 0).get(0));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 1).get(1));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 2).get(2));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 3).get(3));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 4).get(4));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 5).get(5));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 6).get(6));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 7).get(7));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 8).get(8));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 9).get(9));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 10).get(10));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 11).get(11));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 12).get(12));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 13).get(13));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 14).get(14));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 15).get(15));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 16).get(16));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 17).get(17));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 18).get(18));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 19).get(19));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 20).get(20));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 21).get(21));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 22).get(22));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 23).get(23));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 24).get(24));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 25).get(25));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 26).get(26));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 27).get(27));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 28).get(28));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 29).get(29));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 30).get(30));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 31).get(31));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 32).get(32));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 33).get(33));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 34).get(34));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 35).get(35));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 36).get(36));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 37).get(37));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 38).get(38));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 39).get(39));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 40).get(40));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 41).get(41));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 42).get(42));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 43).get(43));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 44).get(44));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 45).get(45));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 46).get(46));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 47).get(47));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 48).get(48));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 49).get(49));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 50).get(50));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 51).get(51));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 52).get(52));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 53).get(53));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 54).get(54));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 55).get(55));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 56).get(56));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 57).get(57));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 58).get(58));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 59).get(59));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 60).get(60));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 61).get(61));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 62).get(62));
        assert!(StaticBitmap::<u64, LSB>::new(1 << 63).get(63));
        assert!(!StaticBitmap::<u64, LSB>::new(0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111).get(64));
        
        // Slice
        assert!(StaticBitmap::<&'static [u8], LSB>::new(&[1u8][..]).get(0));
        assert!(StaticBitmap::<&'static [u8], LSB>::new(&[1u8, 1][..]).get(8));
        assert!(!StaticBitmap::<&'static [u8], LSB>::new(&[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..]).get(999));
        assert!(StaticBitmap::<&'static [u16], LSB>::new(&[1u16][..]).get(0));
        assert!(StaticBitmap::<&'static [u16], LSB>::new(&[1u16, 1u16][..]).get(16));
        assert!(!StaticBitmap::<&'static [u16], LSB>::new(&[0b1111_1111_1111_1111u16, 0b1111_1111_1111_1111, 0b1111_1111_1111_1111][..]).get(999));
        assert!(StaticBitmap::<&'static [u32], LSB>::new(&[1u32][..]).get(0));
        assert!(StaticBitmap::<&'static [u32], LSB>::new(&[1u32, 1][..]).get(32));
        assert!(!StaticBitmap::<&'static [u32], LSB>::new(&[0b1111_1111_1111_1111_1111_1111_1111_1111u32, 0b1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111][..]).get(999));
        assert!(StaticBitmap::<&'static [u64], LSB>::new(&[1u64][..]).get(0));
        assert!(StaticBitmap::<&'static [u64], LSB>::new(&[1u64, 1][..]).get(64));
        assert!(!StaticBitmap::<&'static [u64], LSB>::new(&[0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111u64, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111][..]).get(999));

        let v = &[1u8][..];
        assert!(StaticBitmap::<&[u8], LSB>::new(v).get(0));
        let v = &[1u8, 1][..];
        assert!(StaticBitmap::<&[u8], LSB>::new(v).get(8));
        let v = &[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..];
        assert!(!StaticBitmap::<&[u8], LSB>::new(v).get(999));
        let v = &[1u16][..];
        assert!(StaticBitmap::<&[u16], LSB>::new(v).get(0));
        let v = &[1u16, 1u16][..];
        assert!(StaticBitmap::<&[u16], LSB>::new(v).get(16));
        let v = &[0b1111_1111_1111_1111u16, 0b1111_1111_1111_1111, 0b1111_1111_1111_1111][..];
        assert!(!StaticBitmap::<&[u16], LSB>::new(v).get(999));
        let v = &[1u32][..];
        assert!(StaticBitmap::<&[u32], LSB>::new(v).get(0));
        let v = &[1u32, 1][..];
        assert!(StaticBitmap::<&[u32], LSB>::new(v).get(32));
        let v = &[0b1111_1111_1111_1111_1111_1111_1111_1111u32, 0b1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111][..];
        assert!(!StaticBitmap::<&[u32], LSB>::new(v).get(999));
        let v = &[1u64][..];
        assert!(StaticBitmap::<&[u64], LSB>::new(v).get(0));
        let v = &[1u64, 1][..];
        assert!(StaticBitmap::<&[u64], LSB>::new(v).get(64));
        let v = &[0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111u64, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111][..];
        assert!(!StaticBitmap::<&[u64], LSB>::new(v).get(999));

        // Array
        assert!(StaticBitmap::<[u8; 1], LSB>::new([1; 1]).get(0));
        assert!(StaticBitmap::<[u8; 2], LSB>::new([1; 2]).get(8));
        assert!(!StaticBitmap::<[u8; 3], LSB>::new([0b1111_1111; 3]).get(999));
        assert!(StaticBitmap::<[u16; 1], LSB>::new([1; 1]).get(0));
        assert!(StaticBitmap::<[u16; 2], LSB>::new([1; 2]).get(16));
        assert!(!StaticBitmap::<[u16; 3], LSB>::new([0b1111_1111_1111_1111; 3]).get(999));
        assert!(StaticBitmap::<[u32; 1], LSB>::new([1; 1]).get(0));
        assert!(StaticBitmap::<[u32; 2], LSB>::new([1; 2]).get(32));
        assert!(!StaticBitmap::<[u32; 3], LSB>::new([0b1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));
        assert!(StaticBitmap::<[u64; 1], LSB>::new([1; 1]).get(0));
        assert!(StaticBitmap::<[u64; 2], LSB>::new([1; 2]).get(64));
        assert!(!StaticBitmap::<[u64; 3], LSB>::new([0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));

        // Vec
        assert!(StaticBitmap::<Vec<u8>, LSB>::new(vec![1; 1]).get(0));
        assert!(StaticBitmap::<Vec<u8>, LSB>::new(vec![1; 2]).get(8));
        assert!(!StaticBitmap::<Vec<u8>, LSB>::new(vec![0b1111_1111; 3]).get(999));
        assert!(StaticBitmap::<Vec<u16>, LSB>::new(vec![1; 1]).get(0));
        assert!(StaticBitmap::<Vec<u16>, LSB>::new(vec![1; 2]).get(16));
        assert!(!StaticBitmap::<Vec<u16>, LSB>::new(vec![0b1111_1111_1111_1111; 3]).get(999));
        assert!(StaticBitmap::<Vec<u32>, LSB>::new(vec![1; 1]).get(0));
        assert!(StaticBitmap::<Vec<u32>, LSB>::new(vec![1; 2]).get(32));
        assert!(!StaticBitmap::<Vec<u32>, LSB>::new(vec![0b1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));
        assert!(StaticBitmap::<Vec<u64>, LSB>::new(vec![1; 1]).get(0));
        assert!(StaticBitmap::<Vec<u64>, LSB>::new(vec![1; 2]).get(64));
        assert!(!StaticBitmap::<Vec<u64>, LSB>::new(vec![0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));

        // Bytes
        #[cfg(feature = "bytes")]
        {
            use bytes::{Bytes, BytesMut};
            assert!(StaticBitmap::<Bytes, LSB>::new(Bytes::from_static(&[1])).get(0));
            assert!(StaticBitmap::<Bytes, LSB>::new(Bytes::from_static(&[1, 1])).get(8));
            assert!(!StaticBitmap::<Bytes, LSB>::new(Bytes::from_static(&[0b1111_1111, 0b1111_1111, 0b1111_1111])).get(999));
            assert!(StaticBitmap::<BytesMut, LSB>::new(BytesMut::from(&[1u8][..])).get(0));
            assert!(StaticBitmap::<BytesMut, LSB>::new(BytesMut::from(&[1u8, 1][..])).get(8));
            assert!(!StaticBitmap::<BytesMut, LSB>::new(BytesMut::from(&[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..])).get(999));
        }

        // SmallVec
        #[cfg(feature = "smallvec")]
        {
            use smallvec::SmallVec;
            assert!(StaticBitmap::<SmallVec<[u8; 1]>, LSB>::new(SmallVec::from([1u8])).get(0));
            assert!(StaticBitmap::<SmallVec<[u8; 2]>, LSB>::new(SmallVec::from([1u8, 1])).get(8));
            assert!(!StaticBitmap::<SmallVec<[u8; 3]>, LSB>::new(SmallVec::from([0b1111_1111u8, 0b1111_1111, 0b1111_1111])).get(999));
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
        let mut v = StaticBitmap::<&mut [u8], LSB>::new(inner.as_mut_slice());
        v.set(0, true);
        v.set(15, true);
        assert!(v.try_set(16, true).is_err());
        assert!(v.get(0));
        assert!(v.get(15));

        let mut inner = vec![0, 0];
        let mut v = StaticBitmap::<&mut [u16], LSB>::new(inner.as_mut_slice());
        v.set(0, true);
        v.set(31, true);
        assert!(v.try_set(32, true).is_err());
        assert!(v.get(0));
        assert!(v.get(31));

        let mut inner = vec![0, 0];
        let mut v = StaticBitmap::<&mut [u32], LSB>::new(inner.as_mut_slice());
        v.set(0, true);
        v.set(63, true);
        assert!(v.try_set(64, true).is_err());
        assert!(v.get(0));
        assert!(v.get(63));

        let mut inner = vec![0, 0];
        let mut v = StaticBitmap::<&mut [u64], LSB>::new(inner.as_mut_slice());
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
        let mut v = StaticBitmap::<Vec<u8>, LSB>::new(vec![0, 0]);
        v.set(0, true);
        v.set(15, true);
        assert!(v.try_set(16, true).is_err());
        assert!(v.get(0));
        assert!(v.get(15));

        let mut v = StaticBitmap::<Vec<u16>, LSB>::new(vec![0, 0]);
        v.set(0, true);
        v.set(31, true);
        assert!(v.try_set(32, true).is_err());
        assert!(v.get(0));
        assert!(v.get(31));

        let mut v = StaticBitmap::<Vec<u32>, LSB>::new(vec![0, 0]);
        v.set(0, true);
        v.set(63, true);
        assert!(v.try_set(64, true).is_err());
        assert!(v.get(0));
        assert!(v.get(63));

        let mut v = StaticBitmap::<Vec<u64>, LSB>::new(vec![0, 0]);
        v.set(0, true);
        v.set(127, true);
        assert!(v.try_set(128, true).is_err());
        assert!(v.get(0));
        assert!(v.get(127));

        // Bytes
        #[cfg(feature = "bytes")]
        {
            use bytes::{BytesMut};
            let mut v = StaticBitmap::<BytesMut, LSB>::new(BytesMut::zeroed(2));
            v.set(0, true);
            v.set(15, true);
            assert!(v.try_set(16, true).is_err());
            assert!(v.get(0));
            assert!(v.get(15));
        }
        
        #[cfg(feature = "smallvec")]
        {
            use smallvec::{SmallVec, smallvec};
            let mut v = StaticBitmap::<SmallVec<[u8; 2]>, LSB>::new(smallvec![0, 0]);
            v.set(0, true);
            v.set(15, true);
            assert!(v.try_set(16, true).is_err());
            assert!(v.get(0));
            assert!(v.get(15));
        }
    }
}
