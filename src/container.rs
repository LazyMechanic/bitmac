use crate::ResizeError;

pub trait ContainerMut: AsRef<[u8]> + AsMut<[u8]> {
    /// Resizes the container in-place so that `len` is equal to `new_len`.
    ///
    /// Returns `OK(())` on success and `Err(_)` on failure.
    fn try_resize(&mut self, new_len: usize, value: u8) -> Result<(), ResizeError>;
}

impl ContainerMut for Vec<u8> {
    #[inline]
    fn try_resize(&mut self, new_len: usize, value: u8) -> Result<(), ResizeError> {
        Vec::resize(self, new_len, value);
        Ok(())
    }
}

impl<const N: usize> ContainerMut for [u8; N] {
    #[inline]
    fn try_resize(&mut self, _new_len: usize, _value: u8) -> Result<(), ResizeError> {
        Err(ResizeError::new("array cannot be resized"))
    }
}

#[cfg(feature = "bytes")]
impl ContainerMut for bytes::BytesMut {
    #[inline]
    fn try_resize(&mut self, new_len: usize, value: u8) -> Result<(), ResizeError> {
        bytes::BytesMut::resize(self, new_len, value);
        Ok(())
    }
}

#[cfg(feature = "smallvec")]
impl<A> ContainerMut for smallvec::SmallVec<A>
where
    A: smallvec::Array<Item = u8>,
{
    #[inline]
    fn try_resize(&mut self, new_len: usize, value: u8) -> Result<(), ResizeError> {
        smallvec::SmallVec::resize(self, new_len, value);
        Ok(())
    }
}
