mod c64;
mod d64;
mod error;
mod s64;
mod sc64;

pub use c64::*;
pub use d64::*;
pub use error::Error;
pub use s64::*;
pub use sc64::*;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const DEFAULT_BUF_SIZE: usize = 0x4000;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub const DEFAULT_BUF_SIZE: usize = 0x100000;

#[cfg(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
))]
pub const DEFAULT_BUF_SIZE: usize = 0x1000;

pub const PTRHEADER64: [u8; 8] = [b'P', b'T', b'R', 64, 0, 0, 0, 0];

pub const PTRHEADER32: [u8; 8] = [b'P', b'T', b'R', 32, 0, 0, 0, 0];
