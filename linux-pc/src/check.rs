use std::{fs::File, io, io::Read, path::Path};

use crate::proc64::Page64;

pub const EXE: [[u8; 4]; 1] = [[0x7f, b'E', b'L', b'F']];

#[inline]
pub fn check_region(page: &Page64) -> bool {
    if !page.is_read() {
        return false;
    }

    if matches!(page.name(), "[stack]" | "[heap]") || check_exe(page) || page.name().is_empty() {
        return true;
    }

    false
}

#[inline]
pub fn check_exe(page: &Page64) -> bool {
    let path = Path::new(page.name());
    if !path.exists() || path.starts_with("/dev") {
        return false;
    }

    let mut header = [0; 4];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_or(false, |_| EXE.contains(&header))
}

pub fn check_bitness<P: AsRef<Path>>(path: P) -> io::Result<u8> {
    let mut header = [0; 16];
    File::open(path).and_then(|mut f| f.read_exact(&mut header))?;

    let bit = match header[4] {
        1 => 32,
        2 => 64,
        _ => header[4],
    };
    Ok(bit)
}
