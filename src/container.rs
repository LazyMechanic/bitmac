#[cfg(feature = "bytes")]
use bytes::{Bytes, BytesMut};
#[cfg(feature = "smallvec")]
use smallvec::{Array, SmallVec};

use crate::{number::Number, BitAccess, OutOfBoundsError};

pub trait ContainerRead<B>
where
    B: BitAccess,
{
    type Slot: Number;

    fn get_slot(&self, idx: usize) -> Self::Slot;

    fn slots_count(&self) -> usize;

    fn get_bit(&self, idx: usize) -> bool {
        // If idx out of bounds
        if idx >= self.bits_count() {
            return false;
        }

        let slot_idx = idx / <Self::Slot as Number>::BITS_COUNT;
        let bit_idx = idx - slot_idx * <Self::Slot as Number>::BITS_COUNT;

        B::get(self.get_slot(slot_idx), bit_idx)
    }

    #[inline]
    fn bits_count(&self) -> usize {
        self.slots_count() * <Self::Slot as Number>::BITS_COUNT
    }
}
pub trait ContainerWrite<B>: ContainerRead<B>
where
    B: BitAccess,
{
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot;

    fn try_set_bit(&mut self, idx: usize, val: bool) -> Result<(), OutOfBoundsError> {
        if idx >= self.bits_count() {
            return Err(OutOfBoundsError::new(idx, 0..self.bits_count()));
        }

        let slot_idx = idx / <Self::Slot as Number>::BITS_COUNT;
        let bit_idx = idx - slot_idx * <Self::Slot as Number>::BITS_COUNT;

        let slot = self.get_mut_slot(slot_idx);
        *slot = B::set(*slot, bit_idx, val);
        Ok(())
    }

    #[inline]
    fn set_bit(&mut self, idx: usize, val: bool) {
        let _ = self.try_set_bit(idx, val);
    }
}

impl<N, B> ContainerRead<B> for &'_ [N]
where
    N: Number,
    B: BitAccess,
{
    type Slot = N;

    #[inline]
    fn get_slot(&self, idx: usize) -> Self::Slot {
        self[idx]
    }

    #[inline]
    fn slots_count(&self) -> usize {
        self.len()
    }
}

impl<N, B> ContainerRead<B> for &'_ mut [N]
where
    N: Number,
    B: BitAccess,
{
    type Slot = N;

    #[inline]
    fn get_slot(&self, idx: usize) -> Self::Slot {
        self[idx]
    }

    #[inline]
    fn slots_count(&self) -> usize {
        self.len()
    }
}

impl<N, B> ContainerWrite<B> for &'_ mut [N]
where
    N: Number,
    B: BitAccess,
{
    #[inline]
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        &mut self[idx]
    }
}

impl<N, B> ContainerRead<B> for Box<[N]>
where
    N: Number,
    B: BitAccess,
{
    type Slot = N;

    #[inline]
    fn get_slot(&self, idx: usize) -> Self::Slot {
        self[idx]
    }

    #[inline]
    fn slots_count(&self) -> usize {
        self.len()
    }
}

impl<N, B> ContainerWrite<B> for Box<[N]>
where
    N: Number,
    B: BitAccess,
{
    #[inline]
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        &mut self[idx]
    }
}

impl<N, const LEN: usize, B> ContainerRead<B> for [N; LEN]
where
    N: Number,
    B: BitAccess,
{
    type Slot = N;

    #[inline]
    fn get_slot(&self, idx: usize) -> Self::Slot {
        self[idx]
    }

    #[inline]
    fn slots_count(&self) -> usize {
        self.len()
    }
}

impl<N, const LEN: usize, B> ContainerWrite<B> for [N; LEN]
where
    N: Number,
    B: BitAccess,
{
    #[inline]
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        &mut self[idx]
    }
}

impl<N, B> ContainerRead<B> for Vec<N>
where
    N: Number,
    B: BitAccess,
{
    type Slot = N;

    fn get_slot(&self, idx: usize) -> Self::Slot {
        self[idx]
    }

    fn slots_count(&self) -> usize {
        self.len()
    }
}

impl<N, B> ContainerWrite<B> for Vec<N>
where
    N: Number,
    B: BitAccess,
{
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        &mut self[idx]
    }
}

#[cfg(feature = "smallvec")]
impl<A, N, B> ContainerRead<B> for SmallVec<A>
where
    A: Array<Item = N>,
    N: Number,
    B: BitAccess,
{
    type Slot = N;

    fn get_slot(&self, idx: usize) -> Self::Slot {
        self[idx]
    }

    fn slots_count(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "smallvec")]
impl<A, N, B> ContainerWrite<B> for SmallVec<A>
where
    A: Array<Item = N>,
    N: Number,
    B: BitAccess,
{
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        &mut self[idx]
    }
}

#[cfg(feature = "bytes")]
impl<B> ContainerRead<B> for Bytes
where
    B: BitAccess,
{
    type Slot = u8;

    fn get_slot(&self, idx: usize) -> Self::Slot {
        self[idx]
    }

    fn slots_count(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "bytes")]
impl<B> ContainerRead<B> for BytesMut
where
    B: BitAccess,
{
    type Slot = u8;

    fn get_slot(&self, idx: usize) -> Self::Slot {
        self[idx]
    }

    fn slots_count(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "bytes")]
impl<B> ContainerWrite<B> for BytesMut
where
    B: BitAccess,
{
    fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
        &mut self[idx]
    }
}

macro_rules! container_impl {
    ($ty:ty) => {
        impl<B> ContainerRead<B> for $ty
        where
            B: BitAccess,
        {
            type Slot = $ty;

            #[inline]
            fn get_slot(&self, idx: usize) -> Self::Slot {
                assert!(idx == 0);
                *self
            }

            #[inline]
            fn slots_count(&self) -> usize {
                1
            }

            fn get_bit(&self, idx: usize) -> bool
            where
                B: BitAccess,
            {
                if idx < <Self as Number>::BITS_COUNT {
                    B::get(*self, idx)
                } else {
                    false
                }
            }

            #[inline]
            fn bits_count(&self) -> usize {
                <Self as Number>::BITS_COUNT
            }
        }

        impl<B> ContainerWrite<B> for $ty
        where
            B: BitAccess,
        {
            fn get_mut_slot(&mut self, idx: usize) -> &mut Self::Slot {
                assert!(idx == 0);
                self
            }

            fn try_set_bit(&mut self, idx: usize, val: bool) -> Result<(), OutOfBoundsError> {
                if idx < <Self as Number>::BITS_COUNT {
                    *self = B::set(*self, idx, val);
                    Ok(())
                } else {
                    Err(OutOfBoundsError::new(idx, 0..<Self as Number>::BITS_COUNT))
                }
            }
        }
    };
}

container_impl!(u8);
container_impl!(u16);
container_impl!(u32);
container_impl!(u64);
container_impl!(u128);
