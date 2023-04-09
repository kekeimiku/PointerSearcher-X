use std::{fs::File, io::Read};

use vmmap::VirtualQuery;

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

pub fn check_exe<Q: VirtualQuery>(map: &Q) -> bool {
    if !map.is_read() {
        return false;
    }

    let Some(path) = map.path() else {
    return false;
};

    #[cfg(target_os = "linux")]
    if path.starts_with("/dev") || path.starts_with("/usr") {
        return false;
    }

    #[cfg(target_os = "macos")]
    if path.starts_with("/usr") {
        return false;
    }

    if let Ok(mut file) = File::open(path) {
        let mut buf = [0; 4];
        if file.read_exact(&mut buf).is_ok() {
            return EXE.eq(&buf);
        }
    }
    false
}
