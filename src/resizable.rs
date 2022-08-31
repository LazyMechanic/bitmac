use crate::number::Number;

pub trait Resizable {
    type Slot: Number;

    /// Resizes the `Self` in-place so that `len` is equal to `new_len`.
    fn resize(&mut self, new_len: usize, value: Self::Slot);
}

impl<N> Resizable for Vec<N>
where
    N: Number,
{
    type Slot = N;

    #[inline]
    fn resize(&mut self, new_len: usize, value: Self::Slot) {
        Vec::resize(self, new_len, value);
    }
}

#[cfg(feature = "bytes")]
impl Resizable for bytes::BytesMut {
    type Slot = u8;

    #[inline]
    fn resize(&mut self, new_len: usize, value: Self::Slot) {
        bytes::BytesMut::resize(self, new_len, value);
    }
}

#[cfg(feature = "smallvec")]
impl<A, N> Resizable for smallvec::SmallVec<A>
where
    A: smallvec::Array<Item = N>,
    N: Number,
{
    type Slot = N;

    #[inline]
    fn resize(&mut self, new_len: usize, value: Self::Slot) {
        smallvec::SmallVec::resize(self, new_len, value);
    }
}
