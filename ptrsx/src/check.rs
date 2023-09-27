use vmmap::{macos::VirtualQueryExt, VirtualQuery};

#[cfg(target_os = "linux")]
pub const EXE: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
))]
pub const EXE: [[u8; 4]; 2] = [[0xCA, 0xFE, 0xBA, 0xBE], [0xCF, 0xFA, 0xED, 0xFE]];

#[inline]
pub fn check_region<Q: VirtualQuery + VirtualQueryExt>(page: &Q) -> bool {
    if !page.is_read() {
        return false;
    }

    #[cfg(target_os = "macos")]
    {
        check_exe(page) || page.path().is_none() && matches!(page.tag(), |1..=9| 11 | 30 | 33 | 60 | 61)
    }

    #[cfg(target_os = "linux")]
    {
        matches!(page.name(), "[stack]" | "[heap]") || check_exe(page) || page.name().is_empty()
    }

    #[cfg(target_os = "windows")]
    (check_exe(page) || page.path().is_none())
}

#[cfg(target_os = "macos")]
#[inline]
fn check_exe<Q: VirtualQueryExt>(page: &Q) -> bool {
    use std::{fs::File, io::Read};

    let Some(path) = page.path() else {
        return false;
    };

    if path.starts_with("/usr") {
        return false;
    }

    let mut header = [0; 4];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .is_ok_and(|_| EXE.contains(&header))
}

#[cfg(target_os = "linux")]
#[inline]
pub fn check_exe<Q: VirtualQueryExt>(page: &Q) -> bool {
    let path = std::path::Path::new(page.name());
    if !path.exists() || path.starts_with("/dev") {
        return false;
    }

    let mut header = [0; 4];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .is_ok_and(|_| EXE.eq(&header))
}

#[cfg(target_os = "windows")]
#[inline]
pub fn check_exe<Q: VirtualQuery + VirtualQueryExt>(page: &Q) -> bool {
    let Some(path) = page.path() else {
        return false;
    };

    if path.starts_with("\\Device\\HarddiskVolume3\\Windows\\System32") {
        return false;
    }

    path.extension().is_some_and(|s| s == "dll" || s == "exe")
}
