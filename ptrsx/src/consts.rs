#[cfg(target_os = "linux")]
pub const CHUNK_SIZE: usize = 0x100000;

#[cfg(target_os = "macos")]
pub const CHUNK_SIZE: usize = 0x4000;

pub const POINTER_SIZE: usize = core::mem::size_of::<Address>();

pub const MAX_BUF_SIZE: usize = 0x100000;

pub type Address = usize;

pub const BIN_CONFIG: bincode::config::Configuration = bincode::config::standard();

#[cfg(target_os = "linux")]
pub const EXE: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[cfg(target_os = "macos")]
pub const EXE: [u8; 4] = [0xCF, 0xFA, 0xED, 0xFE];