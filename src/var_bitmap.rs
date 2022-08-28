use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use crate::{
    container::{ContainerRead, ContainerWrite},
    intersection::{try_intersection_impl, try_intersection_in_impl, Intersection, IntersectionIn},
    iter::{IntoIter, Iter},
    number::Number,
    resizable::Resizable,
    resizing_strategy::{FinalLength, MinimumRequiredLength, ResizingStrategy},
    union::{try_union_impl, try_union_in_impl, Union, UnionIn},
    with_slots::TryWithSlots,
    BitAccess, IntersectionError, ResizeError, StaticBitmap, UnionError,
};

#[derive(Default, Clone, Eq, PartialEq)]
pub struct VarBitmap<D, S, B> {
    data: D,
    resizing_strategy: S,
    phantom: PhantomData<B>,
}

impl<D, S, B> VarBitmap<D, S, B> {
    pub fn into_inner(self) -> D {
        self.data
    }

    pub fn as_static(&self) -> StaticBitmap<&D, B> {
        StaticBitmap::from(&self.data)
    }

    pub fn into_static(self) -> StaticBitmap<D, B> {
        StaticBitmap::from(self.data)
    }
}

impl<D, N, S, B> VarBitmap<D, S, B>
where
    D: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    pub fn get(&self, idx: usize) -> bool {
        self.data.get_bit(idx)
    }
}

impl<D, N, S, B> VarBitmap<D, S, B>
where
    D: ContainerWrite<B, Slot = N> + Resizable<Item = N>,
    N: Number,
    S: ResizingStrategy,
    B: BitAccess,
{
    pub fn set(&mut self, idx: usize, val: bool) {
        let _ = self.try_set(idx, val);
    }

    pub fn try_set(&mut self, idx: usize, val: bool) -> Result<(), ResizeError> {
        let max_idx = self.data.bits_count();
        if idx < max_idx {
            self.data.set_bit(idx, val);
        } else {
            // Change state only if set to true
            // Try to resize container
            let old_len = self.data.slots_count();
            let min_req_len = old_len + (idx - max_idx) / N::BITS_COUNT + 1;
            let min_req_len = MinimumRequiredLength(min_req_len);

            // Call .try_resize() if new value is `1` and .try_resize_opt() if new value is `0`
            if val {
                let FinalLength(new_len) =
                    self.resizing_strategy
                        .try_resize(min_req_len, old_len, idx)?;

                // Resize container if new length doesn't match old length
                if new_len != old_len {
                    self.data.resize(new_len, N::ZERO);
                }
                self.data.set_bit(idx, val);
            } else if let Some(FinalLength(new_len)) =
                self.resizing_strategy
                    .try_resize_opt(min_req_len, old_len, idx)?
            {
                // Resize container if new length doesn't match old length
                if new_len != old_len {
                    self.data.resize(new_len, N::ZERO);
                }
                self.data.set_bit(idx, val);
            }
        }

        Ok(())
    }
}

impl<D, S, B> AsRef<D> for VarBitmap<D, S, B> {
    fn as_ref(&self) -> &D {
        &self.data
    }
}

impl<D, S, B> AsMut<D> for VarBitmap<D, S, B> {
    fn as_mut(&mut self) -> &mut D {
        &mut self.data
    }
}
impl<D, S, B> ContainerRead<B> for VarBitmap<D, S, B>
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

impl<D, S, B> ContainerWrite<B> for VarBitmap<D, S, B>
where
    D: ContainerWrite<B>,
    B: BitAccess,
{
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        self.data.get_mut_slot(idx)
    }
}

impl<D, S, N, B> Debug for VarBitmap<D, S, B>
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

impl<D, N, S, B> IntoIterator for VarBitmap<D, S, B>
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

impl<'a, D, N, S, B> IntoIterator for &'a VarBitmap<D, S, B>
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

impl<D, S, B, Rhs, N> IntersectionIn<Rhs, N, B> for VarBitmap<D, S, B>
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

impl<D, S, B, Rhs, N> Intersection<Rhs, N, B> for VarBitmap<D, S, B>
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

impl<D, S, B, Rhs, N> UnionIn<Rhs, N, B> for VarBitmap<D, S, B>
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

impl<D, S, B, Rhs, N> Union<Rhs, N, B> for VarBitmap<D, S, B>
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
mod tests {}
