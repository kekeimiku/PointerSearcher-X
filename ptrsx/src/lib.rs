#[cfg(target_pointer_width = "32")]
panic!("32-bit is not supported.");

mod c64;
mod check;
mod error;
mod ptrd;
mod ptrs;
mod sc64;

pub use c64::*;
pub use check::*;
pub use error::Error;
pub use ptrd::*;
pub use ptrs::*;
pub use sc64::*;

pub const PTRSIZE: usize = core::mem::size_of::<usize>();

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const DEFAULT_BUF_SIZE: usize = 0x4000;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub const DEFAULT_BUF_SIZE: usize = 0x40000;

#[cfg(any(target_os = "windows", all(target_os = "macos", target_arch = "x86_64"),))]
pub const DEFAULT_BUF_SIZE: usize = 0x1000;

pub const PTRHEADER64: [u8; 8] = [b'P', b'T', b'R', 64, 0, 0, 0, 0];

pub const PTRHEADER32: [u8; 8] = [b'P', b'T', b'R', 32, 0, 0, 0, 0];
