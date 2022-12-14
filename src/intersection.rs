use crate::{
    container::{ContainerRead, ContainerWrite},
    number::Number,
    with_slots::TryWithSlots,
    BitAccess, IntersectionError, SmallContainerSizeError,
};

/// Intersection operator (a & b).
pub trait Intersection<Rhs, N, B>
where
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    /// Calculates intersection in-place. Result will be stored in `dst`.
    ///
    /// ## Panic
    ///
    /// Panics if `dst` cannot fit the entire result.
    /// See non-panic function [`try_intersection_in`].
    ///
    /// [`try_intersection_in`]: crate::intersection::Intersection::try_intersection_in
    fn intersection_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst)
    where
        Dst: ContainerWrite<B, Slot = N>;

    /// Calculates intersection in-place. Result will be stored in `dst`.
    ///
    /// Returns `Err(_)` if `dst` cannot fit the entire result.
    fn try_intersection_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst) -> Result<(), IntersectionError>
    where
        Dst: ContainerWrite<B, Slot = N>;

    /// Calculates intersection. Result container will be created with [`try_with_slots`] function.
    ///
    /// ## Panic
    ///
    /// Panics if `Dst` cannot fit the entire result.
    /// See non-panic function [`try_intersection`].
    ///
    /// [`try_intersection`]: crate::intersection::Intersection::try_intersection
    /// [`try_with_slots`]: crate::with_slots::TryWithSlots::try_with_slots
    fn intersection<Dst>(&self, rhs: &Rhs) -> Dst
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots;

    /// Calculates intersection. Result container will be created with [`try_with_slots`] function.
    ///
    /// Returns `Err(_)` if `Dst` cannot fit the entire result.
    ///
    /// [`try_with_slots`]: crate::with_slots::TryWithSlots::try_with_slots
    fn try_intersection<Dst>(&self, rhs: &Rhs) -> Result<Dst, IntersectionError>
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots;

    /// Calculates intersection length - ones count. It doesn't allocate for storing intersection result.
    ///
    /// Useful if you need to create some storage that relies on the number of required bits presented in the bitmap.
    fn intersection_len(&self, rhs: &Rhs) -> usize;
}

pub(crate) fn try_intersection_in_impl<Lhs, Rhs, Dst, N, B>(
    lhs: &Lhs,
    rhs: &Rhs,
    dst: &mut Dst,
) -> Result<(), IntersectionError>
where
    Lhs: ContainerRead<B, Slot = N>,
    Rhs: ContainerRead<B, Slot = N>,
    Dst: ContainerWrite<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    // TODO: shrink size
    let required_dst_len = usize::min(lhs.slots_count(), rhs.slots_count());
    if dst.slots_count() < required_dst_len {
        return Err(SmallContainerSizeError::new(format!(
            "size of container should be >= {}, but handled {}",
            required_dst_len,
            dst.slots_count()
        ))
        .into());
    }
    let max_idx = required_dst_len;

    for i in 0..max_idx {
        let dst_slot = dst.get_mut_slot(i);
        let lhs_slot = lhs.get_slot(i);
        let rhs_slot = rhs.get_slot(i);

        *dst_slot = lhs_slot & rhs_slot;
    }
    Ok(())
}

pub(crate) fn try_intersection_impl<Lhs, Rhs, Dst, N, B>(
    lhs: &Lhs,
    rhs: &Rhs,
) -> Result<Dst, IntersectionError>
where
    Lhs: ContainerRead<B, Slot = N>,
    Rhs: ContainerRead<B, Slot = N>,
    Dst: ContainerWrite<B, Slot = N> + TryWithSlots,
    N: Number,
    B: BitAccess,
{
    // TODO: shrink size
    let slots_count = usize::min(lhs.slots_count(), rhs.slots_count());
    let mut dst = Dst::try_with_slots(slots_count)?;

    try_intersection_in_impl(lhs, rhs, &mut dst)?;
    Ok(dst)
}

pub(crate) fn intersection_len_impl<Lhs, Rhs, N, B>(lhs: &Lhs, rhs: &Rhs) -> usize
where
    Lhs: ContainerRead<B, Slot = N>,
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    let max_idx = usize::min(lhs.slots_count(), rhs.slots_count());

    let mut len = 0;
    for i in 0..max_idx {
        let lhs_slot = lhs.get_slot(i);
        let rhs_slot = rhs.get_slot(i);
        let intersect = lhs_slot & rhs_slot;
        len += intersect.count_ones() as usize;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LSB;

    #[test]
    fn try_intersection_ok() {
        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let exp: u8 = 0b0010_0100;
        assert_eq!(
            try_intersection_impl::<_, _, u8, _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let exp: [u8; 1] = [0b0010_0100];
        assert_eq!(
            try_intersection_impl::<_, _, [u8; 1], _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let exp: Vec<u8> = vec![0b0010_0100];
        assert_eq!(
            try_intersection_impl::<_, _, Vec<u8>, _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: u8 = 0b0010_0100;
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            assert_eq!(
                try_intersection_impl::<_, _, BytesMut, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: u8 = 0b0010_0100;
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_0100];
            assert_eq!(
                try_intersection_impl::<_, _, SmallVec<[u8; 1]>, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }

        /////////

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: u8 = 0b0010_0100;
        assert_eq!(
            try_intersection_impl::<_, _, u8, _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        assert_eq!(
            try_intersection_impl::<_, _, [u8; 1], _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        assert_eq!(
            try_intersection_impl::<_, _, Vec<u8>, _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            assert_eq!(
                try_intersection_impl::<_, _, BytesMut, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_0100];
            assert_eq!(
                try_intersection_impl::<_, _, SmallVec<[u8; 1]>, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }

        /////////

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        assert_eq!(
            try_intersection_impl::<_, _, [u8; 1], _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        assert_eq!(
            try_intersection_impl::<_, _, [u8; 1], _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        assert_eq!(
            try_intersection_impl::<_, _, Vec<u8>, _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            assert_eq!(
                try_intersection_impl::<_, _, BytesMut, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
            let exp: SmallVec<[u8; 2]> = smallvec![0b0010_0100];
            assert_eq!(
                try_intersection_impl::<_, _, SmallVec<[u8; 2]>, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }
    }

    #[test]
    fn try_intersection_err() {
        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        assert!(try_intersection_impl::<_, _, [u8; 10], _, LSB>(&lhs, &rhs).is_err());

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        assert!(try_intersection_impl::<_, _, [u8; 3], _, LSB>(&lhs, &rhs).is_err());
    }

    #[test]
    fn try_intersection_in_ok() {
        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let mut dst: u8 = 0b0000_0000;
        let exp: u8 = 0b0010_0100;
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let v = &mut [0b0000_0000][..];
        let mut dst: &mut [u8] = v;
        let v = &mut [0b0010_0100][..];
        let exp: &mut [u8] = v;
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let mut dst: Vec<u8> = vec![0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: u8 = 0b0010_0100;
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: u8 = 0b0010_0100;
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_0100];
            try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }

        /////////

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: u8 = 0b0000_0000;
        let exp: u8 = 0b0010_0100;
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_0100];
            try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }

        /////////

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_0100];
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_0100];
        try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_0100][..]);
            try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 3] = [0b0010_0100, 0b0110_0000, 0b0000_0000];
            let mut dst: SmallVec<[u8; 2]> = smallvec![0b0000_0000];
            let exp: SmallVec<[u8; 2]> = smallvec![0b0010_0100];
            try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }
    }

    #[test]
    fn try_intersection_in_err() {
        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000; 1];
        assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000; 1];
        assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let v = &mut [0b0000_0000][..];
        let mut dst: &mut [u8] = v;
        assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());
        }

        //////////////////

        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: [u8; 1] = [0b0000_0000; 1];
        assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000; 1];
        assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let v = &mut [0b0000_0000][..];
        let mut dst: &mut [u8] = v;
        assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            assert!(try_intersection_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());
        }
    }

    #[test]
    fn intersection_len() {
        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        assert_eq!(intersection_len_impl::<_, _, _, LSB>(&lhs, &rhs), 2);

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0110;
        assert_eq!(intersection_len_impl::<_, _, _, LSB>(&lhs, &rhs), 2);

        /////////

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        assert_eq!(intersection_len_impl::<_, _, _, LSB>(&lhs, &rhs), 2);

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0101_0000];
        assert_eq!(intersection_len_impl::<_, _, _, LSB>(&lhs, &rhs), 2);
    }
}
