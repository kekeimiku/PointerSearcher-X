#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use apple::Process;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use linux::Process;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::Process;

mod error;
mod load;
mod rangemap;

use core::{mem, slice};
use std::collections::BTreeMap;

pub use error::Error;
pub use load::load_pointer_map_file;
pub use rangemap::{RangeMap, RangeSet};

pub struct PointerMap {
    pub points: Vec<usize>,
    pub map: BTreeMap<usize, Vec<usize>>,
    pub modules: RangeMap<usize, String>,
}

pub(crate) const MAGIC: &[u8; 4] = b"@PTR";
// pub(crate) const ARCH32: u32 = 1;
pub(crate) const ARCH64: u32 = 2;

#[repr(packed)]
pub struct Header {
    pub magic: [u8; 4],
    pub arch: u32,
    pub _r: [u8; 116],
    pub modules_size: u32,
}

impl Header {
    pub const fn count() -> usize {
        mem::size_of::<Self>()
    }

    pub const fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as _, Self::count()) }
    }
}
