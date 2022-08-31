use crate::{
    container::{ContainerRead, ContainerWrite},
    number::Number,
    with_slots::TryWithSlots,
    BitAccess, SmallContainerSizeError, UnionError,
};

/// Union operator (a | b).
pub trait Union<Rhs, N, B>
where
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    /// Calculates union in-place. Result will be stored in `dst`.
    ///
    /// ## Panic
    ///
    /// Panics if `dst` cannot fit the entire result.
    /// See non-panic function [`try_union_in`].
    ///
    /// [`try_union_in`]: crate::union::Union::try_union_in
    fn union_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst)
    where
        Dst: ContainerWrite<B, Slot = N>;

    /// Calculates union in-place. Result will be stored in `dst`.
    ///
    /// Returns `Err(_)` if `dst` cannot fit the entire result.
    fn try_union_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst) -> Result<(), UnionError>
    where
        Dst: ContainerWrite<B, Slot = N>;

    /// Calculates union. Result container will be created with [`try_with_slots`] function.
    ///
    /// ## Panic
    ///
    /// Panics if `Dst` cannot fit the entire result.
    /// See non-panic function [`try_union`].
    ///
    /// [`try_union`]: crate::union::Union::try_union
    /// [`try_with_slots`]: crate::with_slots::TryWithSlots::try_with_slots
    fn union<Dst>(&self, rhs: &Rhs) -> Dst
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots;

    /// Calculates union. Result container will be created with [`try_with_slots`] function.
    ///
    /// Returns `Err(_)` if `Dst` cannot fit the entire result.
    ///
    /// [`try_with_slots`]: crate::with_slots::TryWithSlots::try_with_slots
    fn try_union<Dst>(&self, rhs: &Rhs) -> Result<Dst, UnionError>
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots;

    /// Calculates union length - ones count. It doesn't allocate for storing union result.
    ///
    /// Useful if you need to create some storage that relies on the number of bits presented in the bitmap.
    fn union_len(&self, rhs: &Rhs) -> usize;
}

pub(crate) fn try_union_in_impl<Lhs, Rhs, Dst, N, B>(
    lhs: &Lhs,
    rhs: &Rhs,
    dst: &mut Dst,
) -> Result<(), UnionError>
where
    Lhs: ContainerRead<B, Slot = N>,
    Rhs: ContainerRead<B, Slot = N>,
    Dst: ContainerWrite<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    // TODO: shrink size
    let required_dst_len = usize::max(lhs.slots_count(), rhs.slots_count());
    if dst.slots_count() < required_dst_len {
        return Err(SmallContainerSizeError::new(format!(
            "size of container should be >= {}, but handled {}",
            required_dst_len,
            dst.slots_count()
        ))
        .into());
    }

    let head_max_idx = usize::min(lhs.slots_count(), rhs.slots_count());
    for i in 0..head_max_idx {
        let dst_slot = dst.get_mut_slot(i);
        let lhs_slot = lhs.get_slot(i);
        let rhs_slot = rhs.get_slot(i);

        *dst_slot = lhs_slot | rhs_slot;
    }

    // Clone rest tail
    let tail_max_idx = usize::max(lhs.slots_count(), rhs.slots_count());
    for i in head_max_idx..tail_max_idx {
        let dst_slot = dst.get_mut_slot(i);
        let rest_slot = if lhs.slots_count() >= rhs.slots_count() {
            lhs.get_slot(i)
        } else {
            rhs.get_slot(i)
        };

        *dst_slot = rest_slot
    }

    Ok(())
}

pub(crate) fn try_union_impl<Lhs, Rhs, Dst, N, B>(lhs: &Lhs, rhs: &Rhs) -> Result<Dst, UnionError>
where
    Lhs: ContainerRead<B, Slot = N>,
    Rhs: ContainerRead<B, Slot = N>,
    Dst: ContainerWrite<B, Slot = N> + TryWithSlots,
    N: Number,
    B: BitAccess,
{
    // TODO: shrink size
    let slots_count = usize::max(lhs.slots_count(), rhs.slots_count());
    let mut dst = Dst::try_with_slots(slots_count)?;

    try_union_in_impl(lhs, rhs, &mut dst)?;
    Ok(dst)
}

pub(crate) fn union_len_impl<Lhs, Rhs, N, B>(lhs: &Lhs, rhs: &Rhs) -> usize
where
    Lhs: ContainerRead<B, Slot = N>,
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    let head_max_idx = usize::min(lhs.slots_count(), rhs.slots_count());

    let mut len = 0;
    for i in 0..head_max_idx {
        let lhs_slot = lhs.get_slot(i);
        let rhs_slot = rhs.get_slot(i);
        let intersect = lhs_slot | rhs_slot;
        len += intersect.count_ones() as usize;
    }

    // Counting rest tail
    let tail_max_idx = usize::max(lhs.slots_count(), rhs.slots_count());
    for i in head_max_idx..tail_max_idx {
        let rest_slot = if lhs.slots_count() >= rhs.slots_count() {
            lhs.get_slot(i)
        } else {
            rhs.get_slot(i)
        };

        len += rest_slot.count_ones() as usize;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LSB;

    #[test]
    fn union() {
        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let exp: u8 = 0b0010_1100;
        assert_eq!(try_union_impl::<_, _, u8, _, LSB>(&lhs, &rhs).unwrap(), exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let exp: [u8; 1] = [0b0010_1100];
        assert_eq!(
            try_union_impl::<_, _, [u8; 1], _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let exp: Vec<u8> = vec![0b0010_1100];
        assert_eq!(
            try_union_impl::<_, _, Vec<u8>, _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: u8 = 0b0010_0100;
            let exp: BytesMut = BytesMut::from(&[0b0010_1100][..]);
            assert_eq!(
                try_union_impl::<_, _, BytesMut, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: u8 = 0b0010_0100;
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_1100];
            assert_eq!(
                try_union_impl::<_, _, SmallVec<[u8; 1]>, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }

        /////////

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: [u8; 2] = [0b0010_1100, 0b0000_0000];
        assert_eq!(
            try_union_impl::<_, _, [u8; 2], _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_1100, 0b0000_0000];
        assert_eq!(
            try_union_impl::<_, _, Vec<u8>, _, LSB>(&lhs, &rhs).unwrap(),
            exp
        );

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let exp: BytesMut = BytesMut::from(&[0b0010_1100, 0b0000_0000][..]);
            assert_eq!(
                try_union_impl::<_, _, BytesMut, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_1100, 0b0000_0000];
            assert_eq!(
                try_union_impl::<_, _, SmallVec<[u8; 1]>, _, LSB>(&lhs, &rhs).unwrap(),
                exp
            );
        }
    }

    #[test]
    fn try_union() {
        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        assert!(try_union_impl::<_, _, [u8; 10], _, LSB>(&lhs, &rhs).is_err());

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        assert!(try_union_impl::<_, _, [u8; 3], _, LSB>(&lhs, &rhs).is_err());

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        assert!(try_union_impl::<_, _, u8, _, LSB>(&lhs, &rhs).is_err());
    }

    #[test]
    fn union_in() {
        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let mut dst: u8 = 0b0000_0000;
        let exp: u8 = 0b0010_1100;
        try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let mut dst: [u8; 1] = [0b0000_0000];
        let exp: [u8; 1] = [0b0010_1100];
        try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        let mut dst: Vec<u8> = vec![0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_1100];
        try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: u8 = 0b0010_0100;
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_1100][..]);
            try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: u8 = 0b0010_0100;
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_1100];
            try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }

        /////////

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: [u8; 2] = [0b0000_0000, 0b0000_0000];
        let exp: [u8; 2] = [0b0010_1100, 0b0000_0000];
        try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000, 0b0000_0000];
        let exp: Vec<u8> = vec![0b0010_1100, 0b0000_0000];
        try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
        assert_eq!(dst, exp);

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000, 0b0000_0000][..]);
            let exp: BytesMut = BytesMut::from(&[0b0010_1100, 0b0000_0000][..]);
            try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: u8 = 0b0010_1100;
            let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000, 0b0000_0000];
            let exp: SmallVec<[u8; 1]> = smallvec![0b0010_1100, 0b0000_0000];
            try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).unwrap();
            assert_eq!(dst, exp);
        }
    }

    #[test]
    fn try_union_in() {
        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: [u8; 2] = [0b0000_0000; 2];
        assert!(try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let mut dst: Vec<u8> = vec![0b0000_0000; 2];
        assert!(try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
        let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
        let v = &mut [0b0000_0000, 0b0000_0000][..];
        let mut dst: &mut [u8] = v;
        assert!(try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());

        #[cfg(feature = "bytes")]
        {
            use bytes::BytesMut;
            let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: BytesMut = BytesMut::from(&[0b0000_0000, 0b0000_0000][..]);
            assert!(try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());
        }

        #[cfg(feature = "smallvec")]
        {
            use smallvec::{smallvec, SmallVec};
            let lhs: [u8; 2] = [0b0010_1100, 0b0000_0000];
            let rhs: [u8; 3] = [0b0010_0100, 0b0000_0000, 0b0000_0000];
            let mut dst: SmallVec<[u8; 1]> = smallvec![0b0000_0000; 2];
            assert!(try_union_in_impl::<_, _, _, _, LSB>(&lhs, &rhs, &mut dst).is_err());
        }
    }

    #[test]
    fn union_len() {
        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0100;
        assert_eq!(union_len_impl::<_, _, _, LSB>(&lhs, &rhs), 3);

        let lhs: u8 = 0b0010_1100;
        let rhs: u8 = 0b0010_0110;
        assert_eq!(union_len_impl::<_, _, _, LSB>(&lhs, &rhs), 4);

        /////////

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0000_0000];
        assert_eq!(union_len_impl::<_, _, _, LSB>(&lhs, &rhs), 3);

        let lhs: u8 = 0b0010_1100;
        let rhs: [u8; 2] = [0b0010_0100, 0b0101_0000];
        assert_eq!(union_len_impl::<_, _, _, LSB>(&lhs, &rhs), 5);
    }
}
