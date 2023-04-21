#[cfg(target_os = "linux")]
pub const CHUNK_SIZE: usize = 0x100000;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const CHUNK_SIZE: usize = 0x4000;

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const CHUNK_SIZE: usize = 0x1000;

#[cfg(all(target_os = "windows"))]
pub const CHUNK_SIZE: usize = 0x1000;

pub const POINTER_SIZE: usize = core::mem::size_of::<Address>();

pub const MAX_BUF_SIZE: usize = 0x100000;

pub type Address = usize;

#[cfg(target_os = "linux")]
pub const EXE: [[u8; 4]; 1] = [[0x7f, b'E', b'L', b'F']];

#[cfg(target_os = "macos")]
pub const EXE: [[u8; 4]; 4] = [
    [0xFE, 0xED, 0xFA, 0xCE],
    [0xCE, 0xFA, 0xED, 0xFE],
    [0xCA, 0xFE, 0xBA, 0xBE],
    [0xBE, 0xBA, 0xFE, 0xCA],
];
