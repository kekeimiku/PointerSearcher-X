pub use super::{Error, VirtualMemoryRead, VirtualQuery};

pub mod error;
mod map;
pub mod real;
pub mod snap;
pub mod vec;
pub mod virt;

use self::{
    error::{SnapshotError, VirtProcError},
    map::RangeMap,
};
