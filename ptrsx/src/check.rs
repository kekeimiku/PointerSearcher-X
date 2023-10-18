use std::{fs::File, io::Read, path::Path};

use vmmap::VirtualQuery;

#[cfg(target_os = "macos")]
#[inline(always)]
pub fn check_region<Q: VirtualQuery + vmmap::macos::VirtualQueryExt>(page: &Q) -> bool {
    if !page.is_read() || page.is_reserve() {
        return false;
    }

    let Some(name) = page.name() else {
        return matches!(page.tag(), |1..=9| 11 | 30 | 33 | 60 | 61);
    };
    let path = Path::new(name);
    if path.starts_with("/usr") {
        return false;
    }
    let mut buf = [0; 8];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut buf))
        .is_ok_and(|_| match buf[0..4] {
            [width, 0xfa, 0xed, 0xfe] if width == 0xcf || width == 0xce => true,
            [0xfe, 0xed, 0xfa, width] if width == 0xcf || width == 0xce => true,
            [0xca, 0xfe, 0xba, 0xbe] => u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) < 45,
            _ => false,
        })
}

#[cfg(target_os = "linux")]
#[inline(always)]
pub fn check_region<Q: VirtualQuery>(page: &Q) -> bool {
    if !page.is_read() {
        return false;
    }

    let Some(name) = page.name() else {
        return true;
    };
    if name.eq("[stack]") || name.eq("[heap]") {
        return true;
    }
    if name.get(0..7).is_some_and(|s| s.eq("/memfd:")) {
        return false;
    }
    let path = Path::new(name);
    if !path.has_root() || path.starts_with("/dev") {
        return false;
    }
    let mut buf = [0; 8];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut buf))
        .is_ok_and(|_| [0x7f, b'E', b'L', b'F'].eq(&buf[0..4]))
}

#[cfg(target_os = "windows")]
#[inline(always)]
pub fn check_region<Q: VirtualQuery>(page: &Q) -> bool {
    if !page.is_read() {
        return false;
    }

    let Some(name) = page.name() else {
        return true;
    };
    if name.contains("\\Windows\\System32\\") {
        return false;
    }
    let name = name.replacen(r#"\Device"#, r#"\\?"#, 1);
    let path = Path::new(&name);
    if !path.has_root() {
        return false;
    }
    let mut buf = [0; 8];
    File::open(path)
        .and_then(|mut f| f.read(&mut buf))
        .is_ok_and(|_| [0x4d, 0x5a].eq(&buf[0..2]))
}
