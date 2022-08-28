pub mod bit_access;
pub mod container;
pub mod error;
pub mod intersection;
pub mod iter;
pub mod number;
pub mod resizable;
pub mod resizing_strategy;
pub mod static_bitmap;
pub mod union;
pub mod var_bitmap;
pub mod with_slots;

pub use bit_access::{BitAccess, LSB, MSB};
pub use error::{
    IntersectionError, OutOfBoundsError, ResizeError, SmallContainerSizeError, UnionError,
    WithSlotsError,
};
pub use intersection::{Intersection, IntersectionIn};
pub use resizing_strategy::{FixedStrategy, LimitStrategy, MinimumRequiredStrategy};
pub use static_bitmap::StaticBitmap;
pub use union::{Union, UnionIn};
pub use var_bitmap::VarBitmap;
