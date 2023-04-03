#[cfg(target_os = "linux")]
pub const CHUNK_SIZE: usize = 0x100000;

#[cfg(target_os = "macos")]
pub const CHUNK_SIZE: usize = 0x4000;

#[cfg(target_os = "macos")]
pub const DLL: &str = "dylib";

#[cfg(target_os = "linux")]
pub const DLL: &str = "so";

pub const POINTER_SIZE: usize = core::mem::size_of::<Address>();

pub const MAX_BUF_SIZE: usize = 0x100000;

pub type Address = usize;

pub const BIN_CONFIG: bincode::config::Configuration = bincode::config::standard();
