#[cfg(not(target_os = "macos"))]
pub mod c32;
#[cfg(not(target_os = "macos"))]
pub mod d32;
pub mod s32;

pub mod c64;
pub mod d64;
pub mod s64;

pub mod sc64;

pub mod error;

pub const DEFAULT_BUF_SIZE: usize = 0x100000;

pub const PTRHEADER64: [u8; 8] = [b'P', b'T', b'R', 64, 0, 0, 0, 0];
