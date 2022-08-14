use crate::ResizeError;

pub trait Container: AsRef<[u8]> + AsMut<[u8]> {
    /// Resizes the container in-place so that `len` is equal to `new_len`.
    ///
    /// Returns `OK(())` on success and `Err(_)` on failure.
    fn try_resize(&mut self, new_len: usize, value: u8) -> Result<(), ResizeError>;
}

impl Container for Vec<u8> {
    #[inline]
    fn try_resize(&mut self, new_len: usize, value: u8) -> Result<(), ResizeError> {
        Vec::resize(self, new_len, value);
        Ok(())
    }
}

impl<const N: usize> Container for [u8; N] {
    #[inline]
    fn try_resize(&mut self, _new_len: usize, _value: u8) -> Result<(), ResizeError> {
        Err(ResizeError::new("array cannot be resized"))
    }
}
