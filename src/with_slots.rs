use crate::{number::Number, resizable::Resizable, WithSlotsError};

pub trait TryWithSlots: Sized {
    /// Creates new container with specified slots number.
    fn try_with_slots(len: usize) -> Result<Self, WithSlotsError>;
}

impl<T, N> TryWithSlots for T
where
    T: Default + Resizable<Slot = N> + Sized,
    N: Number,
{
    fn try_with_slots(len: usize) -> Result<Self, WithSlotsError> {
        let mut this = Self::default();
        this.resize(len, N::ZERO);
        Ok(this)
    }
}

impl<N, const LEN: usize> TryWithSlots for [N; LEN]
where
    N: Number,
{
    fn try_with_slots(len: usize) -> Result<Self, WithSlotsError> {
        if len == LEN {
            Ok([N::ZERO; LEN])
        } else {
            Err(WithSlotsError::new(format!(
                "array can only store {} slots, but handled {}",
                LEN, len
            )))
        }
    }
}

macro_rules! with_slots_impl {
    ($ty:ty) => {
        impl TryWithSlots for $ty {
            fn try_with_slots(len: usize) -> Result<Self, WithSlotsError> {
                if len == 1 {
                    Ok(<$ty as Number>::ZERO)
                } else {
                    Err(WithSlotsError::new(format!(
                        "number can only store 1 slot, but handled {}",
                        len
                    )))
                }
            }
        }
    };
}

with_slots_impl!(u8);
with_slots_impl!(u16);
with_slots_impl!(u32);
with_slots_impl!(u64);
with_slots_impl!(u128);
