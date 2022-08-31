use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use crate::{
    container::{ContainerRead, ContainerWrite},
    grow_strategy::{FinalLength, GrowStrategy, MinimumRequiredLength},
    intersection::{
        intersection_len_impl, try_intersection_impl, try_intersection_in_impl, Intersection,
    },
    iter::{IntoIter, Iter},
    number::Number,
    resizable::Resizable,
    union::{try_union_impl, try_union_in_impl, union_len_impl, Union},
    with_slots::TryWithSlots,
    BitAccess, IntersectionError, ResizeError, StaticBitmap, UnionError,
};

/// A bitmap that can be resized by custom resizing strategy.
///
/// Any structure that implements the [`ContainerRead`] (for read-only access) and
/// [`ContainerWrite`] + [`Resizable`] (for mutable access) traits can be a container of bitmap (e.g. `Vec<T>`).
///
/// It has the same interface as `StaticBitmap` except that mutable access requires resizable container.
/// Container tries to grow if the changing bit is out of bounds.
///
/// Usage example:
/// ```
/// # fn main() {
/// use bitmac::{VarBitmap, LSB, MinimumRequiredStrategy};
///
/// // You can directly check every single bit
/// let bitmap = VarBitmap::<_, LSB, MinimumRequiredStrategy>::from_container(vec![0b0000_0001u8]);
/// assert!(bitmap.get(0));
/// assert!(!bitmap.get(11));
/// assert!(!bitmap.get(13));
///
/// // You can iterate over bits
/// let bitmap = VarBitmap::<_, LSB, MinimumRequiredStrategy>::from_container(vec![0b0000_1001u8, 0b0000_1000]);
/// let mut iter = bitmap.iter().by_bits().enumerate();
/// assert_eq!(iter.next(), Some((0, true)));
/// assert_eq!(iter.next(), Some((1, false)));
/// assert_eq!(iter.next(), Some((2, false)));
/// assert_eq!(iter.next(), Some((3, true)));
/// assert_eq!(iter.next(), Some((4, false)));
///
/// // You can check multiple bits at the same time through the intersection
/// use bitmac::Intersection;
/// let bitmap = VarBitmap::<_, LSB, MinimumRequiredStrategy>::from_container(vec![0b0000_1001u8, 0b0000_1000]);
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
/// // You can directly change every bit
/// let mut bitmap = VarBitmap::<_, LSB, MinimumRequiredStrategy>::from_container(vec![0b0000_1001u8, 0b0001_1000]);
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
/// // If you change the bit exceeding container's length and new bit state is `1` (`true`)
/// // then the container will automatically grow
/// bitmap.set(127, true);
/// assert!(bitmap.get(127));
/// assert_eq!(bitmap.as_ref().len(), 16);
/// # }
/// ```
#[derive(Default, Clone, Eq, PartialEq)]
pub struct VarBitmap<D, B, S> {
    data: D,
    resizing_strategy: S,
    phantom: PhantomData<B>,
}

impl<D, B, S, N> VarBitmap<D, B, S>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
    S: GrowStrategy,
    N: Number,
{
    /// Creates new bitmap from container with specified strategy.
    pub fn new(data: D, resizing_strategy: S) -> Self {
        Self {
            data,
            resizing_strategy,
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

impl<D, B, S, N> VarBitmap<D, B, S>
where
    D: ContainerRead<B, Slot = N> + Default,
    B: BitAccess,
    S: GrowStrategy,
    N: Number,
{
    /// Creates default bitmap with specified strategy.
    pub fn with_resizing_strategy(resizing_strategy: S) -> Self {
        Self {
            data: Default::default(),
            resizing_strategy,
            phantom: Default::default(),
        }
    }
}

impl<D, B, S, N> VarBitmap<D, B, S>
where
    D: ContainerRead<B, Slot = N>,
    B: BitAccess,
    S: GrowStrategy + Default,
    N: Number,
{
    /// Creates new bitmap from container with default strategy.
    pub fn from_container(data: D) -> Self {
        Self {
            data,
            resizing_strategy: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<D, B, S> VarBitmap<D, B, S> {
    /// Converts bitmap into inner container.
    pub fn into_inner(self) -> D {
        self.data
    }
}

impl<D, B, S, N> VarBitmap<D, B, S>
where
    D: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    /// Represents bitmap as static bitmap over `&D` container.
    pub fn as_static<'a>(&'a self) -> StaticBitmap<&'a D, B>
    where
        &'a D: ContainerRead<B>,
    {
        StaticBitmap::from(&self.data)
    }

    /// Converts bitmap into static bitmap.
    pub fn into_static(self) -> StaticBitmap<D, B> {
        StaticBitmap::from(self.data)
    }
}

impl<D, B, S> VarBitmap<D, B, S>
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
    /// // Out of bounds bits always returns false
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

impl<D, B, S, N> VarBitmap<D, B, S>
where
    D: ContainerWrite<B, Slot = N> + Resizable<Slot = N>,
    N: Number,
    S: GrowStrategy,
    B: BitAccess,
{
    /// Sets new state for a single bit.
    ///
    /// ## Panic
    ///
    /// Panics if resizing fails.
    /// See non-panic function [`try_set`].
    ///
    /// ## Usage example:
    /// ```
    /// use bitmac::{VarBitmap, LSB, MinimumRequiredStrategy, LimitStrategy};
    ///
    /// let mut bitmap = VarBitmap::<_, LSB, LimitStrategy<MinimumRequiredStrategy>>::new(
    ///     vec![0u8; 1], LimitStrategy{ strategy: Default::default(), limit: 3 },
    /// );
    /// bitmap.set(6, true);
    /// assert!(bitmap.get(6));
    /// bitmap.set(13, true);
    /// assert!(bitmap.get(13));
    /// bitmap.set(13, false);
    /// assert!(!bitmap.get(13));
    /// // bitmap.set(128, false); <-- Panics
    /// ```
    ///
    /// [`try_set`]: crate::var_bitmap::VarBitmap::try_set
    pub fn set(&mut self, idx: usize, val: bool) {
        self.try_set(idx, val).unwrap();
    }

    /// Sets new state for a single bit.
    ///
    /// Returns `Err(_)` if resizing fails.
    ///
    /// ## Usage example:
    /// ```
    /// use bitmac::{VarBitmap, LSB, MinimumRequiredStrategy, LimitStrategy};
    ///
    /// let mut bitmap = VarBitmap::<_, LSB, LimitStrategy<MinimumRequiredStrategy>>::new(
    ///     vec![0u8; 1], LimitStrategy{ strategy: Default::default(), limit: 3 },
    /// );
    /// assert!(bitmap.try_set(12, true).is_ok());
    /// assert!(bitmap.get(12));
    /// assert_eq!(bitmap.as_ref().len(), 2);
    /// assert!(bitmap.try_set(12, false).is_ok());
    /// assert!(!bitmap.get(12));
    /// assert_eq!(bitmap.as_ref().len(), 2);
    /// // Grow strategy returns error
    /// assert!(bitmap.try_set(128, true).is_err());
    /// assert!(!bitmap.get(128));
    /// assert_eq!(bitmap.as_ref().len(), 2);
    /// ```
    pub fn try_set(&mut self, idx: usize, val: bool) -> Result<(), ResizeError> {
        let max_idx = self.data.bits_count();
        if idx < max_idx {
            self.data.set_bit_unchecked(idx, val);
        } else {
            // Try to resize container
            let old_len = self.data.slots_count();
            let min_req_len = old_len + (idx - max_idx) / N::BITS_COUNT + 1;
            let min_req_len = MinimumRequiredLength(min_req_len);

            // Call .try_resize() if new value is `1` or if strategy supports force resizing
            if val || self.resizing_strategy.is_force_grow() {
                let FinalLength(new_len) =
                    self.resizing_strategy.try_grow(min_req_len, old_len, idx)?;

                // Resize container if new length doesn't match old length
                if new_len != old_len {
                    self.data.resize(new_len, N::ZERO);
                }
                self.data.set_bit_unchecked(idx, val);
            }
        }

        Ok(())
    }
}

impl<D, N, B, S> From<D> for VarBitmap<D, B, S>
where
    D: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
    S: Default,
{
    fn from(f: D) -> Self {
        Self {
            data: f,
            resizing_strategy: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<D, B, S> AsRef<D> for VarBitmap<D, B, S> {
    fn as_ref(&self) -> &D {
        &self.data
    }
}

impl<D, B, S> AsMut<D> for VarBitmap<D, B, S> {
    fn as_mut(&mut self) -> &mut D {
        &mut self.data
    }
}
impl<D, B, S> ContainerRead<B> for VarBitmap<D, B, S>
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

impl<D, B, S> ContainerWrite<B> for VarBitmap<D, B, S>
where
    D: ContainerWrite<B>,
    B: BitAccess,
{
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        self.data.get_mut_slot(idx)
    }
}

impl<D, B, S, N> Debug for VarBitmap<D, B, S>
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

impl<D, B, S> IntoIterator for VarBitmap<D, B, S>
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

impl<'a, D, B, S> IntoIterator for &'a VarBitmap<D, B, S>
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

impl<D, B, S, Rhs, N> Intersection<Rhs, N, B> for VarBitmap<D, B, S>
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

impl<D, B, S, Rhs, N> Union<Rhs, N, B> for VarBitmap<D, B, S>
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
    use crate::{MinimumRequiredStrategy, LSB};

    #[test]
    #[rustfmt::skip]
    fn get_bit() {
        // Number
        assert!(VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(1 << 0).get(0));
        assert!(VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(1 << 1).get(1));
        assert!(VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(1 << 2).get(2));
        assert!(VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(1 << 3).get(3));
        assert!(VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(1 << 4).get(4));
        assert!(VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(1 << 5).get(5));
        assert!(VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(1 << 6).get(6));
        assert!(VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(1 << 7).get(7));
        assert!(!VarBitmap::<u8, LSB, MinimumRequiredStrategy>::from_container(0b1111_1111).get(8));

        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 0).get(0));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 1).get(1));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 2).get(2));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 3).get(3));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 4).get(4));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 5).get(5));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 6).get(6));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 7).get(7));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 8).get(8));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 9).get(9));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 10).get(10));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 11).get(11));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 12).get(12));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 13).get(13));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 14).get(14));
        assert!(VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(1 << 15).get(15));
        assert!(!VarBitmap::<u16, LSB, MinimumRequiredStrategy>::from_container(0b1111_1111_1111_1111).get(16));

        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 0).get(0));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 1).get(1));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 2).get(2));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 3).get(3));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 4).get(4));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 5).get(5));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 6).get(6));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 7).get(7));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 8).get(8));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 9).get(9));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 10).get(10));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 11).get(11));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 12).get(12));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 13).get(13));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 14).get(14));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 15).get(15));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 16).get(16));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 17).get(17));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 18).get(18));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 19).get(19));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 20).get(20));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 21).get(21));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 22).get(22));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 23).get(23));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 24).get(24));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 25).get(25));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 26).get(26));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 27).get(27));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 28).get(28));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 29).get(29));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 30).get(30));
        assert!(VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(1 << 31).get(31));
        assert!(!VarBitmap::<u32, LSB, MinimumRequiredStrategy>::from_container(0b0000_0000_0000_0000_0000_0000_0000_0000).get(32));

        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 0).get(0));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 1).get(1));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 2).get(2));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 3).get(3));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 4).get(4));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 5).get(5));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 6).get(6));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 7).get(7));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 8).get(8));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 9).get(9));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 10).get(10));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 11).get(11));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 12).get(12));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 13).get(13));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 14).get(14));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 15).get(15));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 16).get(16));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 17).get(17));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 18).get(18));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 19).get(19));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 20).get(20));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 21).get(21));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 22).get(22));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 23).get(23));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 24).get(24));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 25).get(25));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 26).get(26));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 27).get(27));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 28).get(28));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 29).get(29));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 30).get(30));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 31).get(31));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 32).get(32));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 33).get(33));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 34).get(34));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 35).get(35));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 36).get(36));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 37).get(37));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 38).get(38));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 39).get(39));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 40).get(40));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 41).get(41));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 42).get(42));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 43).get(43));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 44).get(44));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 45).get(45));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 46).get(46));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 47).get(47));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 48).get(48));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 49).get(49));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 50).get(50));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 51).get(51));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 52).get(52));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 53).get(53));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 54).get(54));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 55).get(55));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 56).get(56));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 57).get(57));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 58).get(58));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 59).get(59));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 60).get(60));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 61).get(61));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 62).get(62));
        assert!(VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(1 << 63).get(63));
        assert!(!VarBitmap::<u64, LSB, MinimumRequiredStrategy>::from_container(0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111).get(64));

        // Slice
        assert!(VarBitmap::<&'static [u8], LSB, MinimumRequiredStrategy>::from_container(&[1u8][..]).get(0));
        assert!(VarBitmap::<&'static [u8], LSB, MinimumRequiredStrategy>::from_container(&[1u8, 1][..]).get(8));
        assert!(!VarBitmap::<&'static [u8], LSB, MinimumRequiredStrategy>::from_container(&[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..]).get(999));
        assert!(VarBitmap::<&'static [u16], LSB, MinimumRequiredStrategy>::from_container(&[1u16][..]).get(0));
        assert!(VarBitmap::<&'static [u16], LSB, MinimumRequiredStrategy>::from_container(&[1u16, 1u16][..]).get(16));
        assert!(!VarBitmap::<&'static [u16], LSB, MinimumRequiredStrategy>::from_container(&[0b1111_1111_1111_1111u16, 0b1111_1111_1111_1111, 0b1111_1111_1111_1111][..]).get(999));
        assert!(VarBitmap::<&'static [u32], LSB, MinimumRequiredStrategy>::from_container(&[1u32][..]).get(0));
        assert!(VarBitmap::<&'static [u32], LSB, MinimumRequiredStrategy>::from_container(&[1u32, 1][..]).get(32));
        assert!(!VarBitmap::<&'static [u32], LSB, MinimumRequiredStrategy>::from_container(&[0b1111_1111_1111_1111_1111_1111_1111_1111u32, 0b1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111][..]).get(999));
        assert!(VarBitmap::<&'static [u64], LSB, MinimumRequiredStrategy>::from_container(&[1u64][..]).get(0));
        assert!(VarBitmap::<&'static [u64], LSB, MinimumRequiredStrategy>::from_container(&[1u64, 1][..]).get(64));
        assert!(!VarBitmap::<&'static [u64], LSB, MinimumRequiredStrategy>::from_container(&[0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111u64, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111][..]).get(999));

        let v = &[1u8][..];
        assert!(VarBitmap::<&[u8], LSB, MinimumRequiredStrategy>::from_container(v).get(0));
        let v = &[1u8, 1][..];
        assert!(VarBitmap::<&[u8], LSB, MinimumRequiredStrategy>::from_container(v).get(8));
        let v = &[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..];
        assert!(!VarBitmap::<&[u8], LSB, MinimumRequiredStrategy>::from_container(v).get(999));
        let v = &[1u16][..];
        assert!(VarBitmap::<&[u16], LSB, MinimumRequiredStrategy>::from_container(v).get(0));
        let v = &[1u16, 1u16][..];
        assert!(VarBitmap::<&[u16], LSB, MinimumRequiredStrategy>::from_container(v).get(16));
        let v = &[0b1111_1111_1111_1111u16, 0b1111_1111_1111_1111, 0b1111_1111_1111_1111][..];
        assert!(!VarBitmap::<&[u16], LSB, MinimumRequiredStrategy>::from_container(v).get(999));
        let v = &[1u32][..];
        assert!(VarBitmap::<&[u32], LSB, MinimumRequiredStrategy>::from_container(v).get(0));
        let v = &[1u32, 1][..];
        assert!(VarBitmap::<&[u32], LSB, MinimumRequiredStrategy>::from_container(v).get(32));
        let v = &[0b1111_1111_1111_1111_1111_1111_1111_1111u32, 0b1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111][..];
        assert!(!VarBitmap::<&[u32], LSB, MinimumRequiredStrategy>::from_container(v).get(999));
        let v = &[1u64][..];
        assert!(VarBitmap::<&[u64], LSB, MinimumRequiredStrategy>::from_container(v).get(0));
        let v = &[1u64, 1][..];
        assert!(VarBitmap::<&[u64], LSB, MinimumRequiredStrategy>::from_container(v).get(64));
        let v = &[0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111u64, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111, 0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111][..];
        assert!(!VarBitmap::<&[u64], LSB, MinimumRequiredStrategy>::from_container(v).get(999));

        // Array
        assert!(VarBitmap::<[u8; 1], LSB, MinimumRequiredStrategy>::from_container([1; 1]).get(0));
        assert!(VarBitmap::<[u8; 2], LSB, MinimumRequiredStrategy>::from_container([1; 2]).get(8));
        assert!(!VarBitmap::<[u8; 3], LSB, MinimumRequiredStrategy>::from_container([0b1111_1111; 3]).get(999));
        assert!(VarBitmap::<[u16; 1], LSB, MinimumRequiredStrategy>::from_container([1; 1]).get(0));
        assert!(VarBitmap::<[u16; 2], LSB, MinimumRequiredStrategy>::from_container([1; 2]).get(16));
        assert!(!VarBitmap::<[u16; 3], LSB, MinimumRequiredStrategy>::from_container([0b1111_1111_1111_1111; 3]).get(999));
        assert!(VarBitmap::<[u32; 1], LSB, MinimumRequiredStrategy>::from_container([1; 1]).get(0));
        assert!(VarBitmap::<[u32; 2], LSB, MinimumRequiredStrategy>::from_container([1; 2]).get(32));
        assert!(!VarBitmap::<[u32; 3], LSB, MinimumRequiredStrategy>::from_container([0b1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));
        assert!(VarBitmap::<[u64; 1], LSB, MinimumRequiredStrategy>::from_container([1; 1]).get(0));
        assert!(VarBitmap::<[u64; 2], LSB, MinimumRequiredStrategy>::from_container([1; 2]).get(64));
        assert!(!VarBitmap::<[u64; 3], LSB, MinimumRequiredStrategy>::from_container([0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));

        // Vec
        assert!(VarBitmap::<Vec<u8>, LSB, MinimumRequiredStrategy>::from_container(vec![1; 1]).get(0));
        assert!(VarBitmap::<Vec<u8>, LSB, MinimumRequiredStrategy>::from_container(vec![1; 2]).get(8));
        assert!(!VarBitmap::<Vec<u8>, LSB, MinimumRequiredStrategy>::from_container(vec![0b1111_1111; 3]).get(999));
        assert!(VarBitmap::<Vec<u16>, LSB, MinimumRequiredStrategy>::from_container(vec![1; 1]).get(0));
        assert!(VarBitmap::<Vec<u16>, LSB, MinimumRequiredStrategy>::from_container(vec![1; 2]).get(16));
        assert!(!VarBitmap::<Vec<u16>, LSB, MinimumRequiredStrategy>::from_container(vec![0b1111_1111_1111_1111; 3]).get(999));
        assert!(VarBitmap::<Vec<u32>, LSB, MinimumRequiredStrategy>::from_container(vec![1; 1]).get(0));
        assert!(VarBitmap::<Vec<u32>, LSB, MinimumRequiredStrategy>::from_container(vec![1; 2]).get(32));
        assert!(!VarBitmap::<Vec<u32>, LSB, MinimumRequiredStrategy>::from_container(vec![0b1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));
        assert!(VarBitmap::<Vec<u64>, LSB, MinimumRequiredStrategy>::from_container(vec![1; 1]).get(0));
        assert!(VarBitmap::<Vec<u64>, LSB, MinimumRequiredStrategy>::from_container(vec![1; 2]).get(64));
        assert!(!VarBitmap::<Vec<u64>, LSB, MinimumRequiredStrategy>::from_container(vec![0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111; 3]).get(999));

        // Bytes
        #[cfg(feature = "bytes")]
        {
            use bytes::{Bytes, BytesMut};
            assert!(VarBitmap::<Bytes, LSB, MinimumRequiredStrategy>::from_container(Bytes::from_static(&[1])).get(0));
            assert!(VarBitmap::<Bytes, LSB, MinimumRequiredStrategy>::from_container(Bytes::from_static(&[1, 1])).get(8));
            assert!(!VarBitmap::<Bytes, LSB, MinimumRequiredStrategy>::from_container(Bytes::from_static(&[0b1111_1111, 0b1111_1111, 0b1111_1111])).get(999));
            assert!(VarBitmap::<BytesMut, LSB, MinimumRequiredStrategy>::from_container(BytesMut::from(&[1u8][..])).get(0));
            assert!(VarBitmap::<BytesMut, LSB, MinimumRequiredStrategy>::from_container(BytesMut::from(&[1u8, 1][..])).get(8));
            assert!(!VarBitmap::<BytesMut, LSB, MinimumRequiredStrategy>::from_container(BytesMut::from(&[0b1111_1111u8, 0b1111_1111, 0b1111_1111][..])).get(999));
        }

        // SmallVec
        #[cfg(feature = "smallvec")]
        {
            use smallvec::SmallVec;
            assert!(VarBitmap::<SmallVec<[u8; 1]>, LSB, MinimumRequiredStrategy>::from_container(SmallVec::from([1u8])).get(0));
            assert!(VarBitmap::<SmallVec<[u8; 2]>, LSB, MinimumRequiredStrategy>::from_container(SmallVec::from([1u8, 1])).get(8));
            assert!(!VarBitmap::<SmallVec<[u8; 3]>, LSB, MinimumRequiredStrategy>::from_container(SmallVec::from([0b1111_1111u8, 0b1111_1111, 0b1111_1111])).get(999));
        }
    }

    #[test]
    #[rustfmt::skip]
    fn set_bit() {
        // Vec
        let mut v = VarBitmap::<Vec<u8>, LSB, MinimumRequiredStrategy>::from_container(vec![0, 0]);
        v.set(0, true);
        v.set(15, true);
        v.set(16, true);
        assert!(v.get(0));
        assert!(v.get(15));
        assert!(v.get(16));

        let mut v = VarBitmap::<Vec<u16>, LSB, MinimumRequiredStrategy>::from_container(vec![0, 0]);
        v.set(0, true);
        v.set(31, true);
        v.set(32, true);
        assert!(v.get(0));
        assert!(v.get(31));
        assert!(v.get(32));

        let mut v = VarBitmap::<Vec<u32>, LSB, MinimumRequiredStrategy>::from_container(vec![0, 0]);
        v.set(0, true);
        v.set(63, true);
        v.set(64, true);
        assert!(v.get(0));
        assert!(v.get(63));
        assert!(v.get(64));

        let mut v = VarBitmap::<Vec<u64>, LSB, MinimumRequiredStrategy>::from_container(vec![0, 0]);
        v.set(0, true);
        v.set(127, true);
        v.set(128, true);
        assert!(v.get(0));
        assert!(v.get(127));
        assert!(v.get(128));

        // Bytes
        #[cfg(feature = "bytes")]
        {
            use bytes::{BytesMut};
            let mut v = VarBitmap::<BytesMut, LSB, MinimumRequiredStrategy>::from_container(BytesMut::zeroed(2));
            v.set(0, true);
            v.set(15, true);
            v.set(16, true);
            assert!(v.get(0));
            assert!(v.get(15));
            assert!(v.get(16));
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{SmallVec, smallvec};
            let mut v = VarBitmap::<SmallVec<[u8; 2]>, LSB, MinimumRequiredStrategy>::from_container(smallvec![0, 0]);
            v.set(0, true);
            v.set(15, true);
            v.set(16, true);
            assert!(v.get(0));
            assert!(v.get(15));
            assert!(v.get(16));
        }
    }
}
