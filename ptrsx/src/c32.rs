#[cfg(target_os = "linux")]
use std::{fs::File, io::Read};

#[cfg(target_os = "linux")]
pub const EXE: [u8; 4] = [0x7f, b'E', b'L', b'F'];

use vmmap::vmmap32::VirtualQuery;
#[cfg(target_os = "linux")]
use vmmap::vmmap32::VirtualQueryExt;
#[cfg(target_os = "windows")]
use vmmap::vmmap32::VirtualQueryExt;

#[inline]
pub fn check_region<Q: VirtualQuery + VirtualQueryExt>(page: &Q) -> bool {
    if !page.is_read() {
        return false;
    }

    #[cfg(target_os = "linux")]
    if matches!(page.name(), "[stack]" | "[heap]") || check_exe(page) || page.name().is_empty() {
        return true;
    }

    false
}

#[cfg(target_os = "linux")]
#[inline]
pub fn check_exe<Q: VirtualQuery + VirtualQueryExt>(page: &Q) -> bool {
    let path = std::path::Path::new(page.name());
    if !path.exists() || path.starts_with("/dev") {
        return false;
    }

    let mut header = [0; 4];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_or(false, |_| EXE.eq(&header))
}
