use crate::{
    container::{ContainerRead, ContainerWrite},
    number::Number,
    with_slots::TryWithSlots,
    BitAccess, IntersectionError, SmallContainerSizeError,
};

pub trait IntersectionIn<Rhs, N, B>
where
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    fn intersection_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst)
    where
        Dst: ContainerWrite<B, Slot = N>;

    fn try_intersection_in<Dst>(&self, rhs: &Rhs, dst: &mut Dst) -> Result<(), IntersectionError>
    where
        Dst: ContainerWrite<B, Slot = N>;
}

pub trait Intersection<Rhs, N, B>: IntersectionIn<Rhs, N, B>
where
    Rhs: ContainerRead<B, Slot = N>,
    N: Number,
    B: BitAccess,
{
    fn intersection<Dst>(&self, rhs: &Rhs) -> Dst
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots;

    fn try_intersection<Dst>(&self, rhs: &Rhs) -> Result<Dst, IntersectionError>
    where
        Dst: ContainerWrite<B, Slot = N> + TryWithSlots;
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
