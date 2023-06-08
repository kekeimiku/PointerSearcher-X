#[cfg(not(target_os = "macos"))]
pub mod c32;
#[cfg(not(target_os = "macos"))]
pub mod d32;
pub mod s32;

pub mod c64;
pub mod d64;
pub mod s64;

pub const DEFAULT_BUF_SIZE: usize = 0x100000;
