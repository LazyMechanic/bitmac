use crate::number::Number;

pub trait Resizable {
    type Item: Number;

    fn resize(&mut self, new_len: usize, value: Self::Item);
}

impl<N> Resizable for Vec<N>
where
    N: Number,
{
    type Item = N;

    #[inline]
    fn resize(&mut self, new_len: usize, value: Self::Item) {
        Vec::resize(self, new_len, value);
    }
}

#[cfg(feature = "bytes")]
impl Resizable for bytes::BytesMut {
    type Item = u8;

    #[inline]
    fn resize(&mut self, new_len: usize, value: Self::Item) {
        bytes::BytesMut::resize(self, new_len, value);
    }
}

#[cfg(feature = "smallvec")]
impl<A, N> Resizable for smallvec::SmallVec<A>
where
    A: smallvec::Array<Item = N>,
    N: Number,
{
    type Item = N;

    #[inline]
    fn resize(&mut self, new_len: usize, value: Self::Item) {
        smallvec::SmallVec::resize(self, new_len, value);
    }
}
