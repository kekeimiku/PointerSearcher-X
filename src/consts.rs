#[cfg(target_os = "linux")]
pub const CHUNK_SIZE: usize = 0x100000;

#[cfg(target_os = "macos")]
pub const CHUNK_SIZE: usize = 0x4000;

#[cfg(target_os = "macos")]
pub const DLL: &str = ".dylib";

#[cfg(target_os = "linux")]
pub const DLL: &str = ".so";

pub const POINTER_SIZE: usize = 8;

pub const LV1_OUT_SIZE: usize = 32;

pub const LV2_OUT_SIZE: usize = 48;

pub const MAX_DEPTH: u8 = 13;

pub const MAX_BUF_SIZE: usize = 0x100000;

pub type Address = usize;
