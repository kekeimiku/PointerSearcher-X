use std::{fs::File, io, io::Read};

#[cfg(target_os = "linux")]
pub const EXE: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const EXE: [[u8; 4]; 2] = [[0xCA, 0xFE, 0xBA, 0xBE], [0xCF, 0xFA, 0xED, 0xFE]];

#[cfg(target_os = "linux")]
use vmmap::vmmap64::VirtualQueryExt;
#[cfg(target_os = "macos")]
use vmmap::vmmap64::VirtualQueryExt;
use vmmap::vmmap64::{ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::d64::create_pointer_map_writer;

pub struct Page<'a> {
    pub start: u64,
    pub end: u64,
    pub path: &'a str,
}

#[cfg(target_os = "macos")]
impl<'a, V> From<&'a V> for Page<'a>
where
    V: VirtualQuery + VirtualQueryExt,
{
    fn from(value: &'a V) -> Self {
        Self {
            start: value.start(),
            end: value.end(),
            path: value.path().and_then(|s| s.to_str()).unwrap_or_else(|| "~err"),
        }
    }
}

#[cfg(target_os = "linux")]
impl<'a, V> From<&'a V> for Page<'a>
where
    V: VirtualQuery + VirtualQueryExt,
{
    fn from(value: &'a V) -> Self {
        Self { start: value.start(), end: value.end(), path: value.name() }
    }
}

#[inline]
pub fn check_region<Q: VirtualQuery + VirtualQueryExt>(page: &Q) -> bool {
    if !page.is_read() {
        return false;
    }

    #[cfg(target_os = "macos")]
    if check_exe(page) || page.path().is_none() && matches!(page.tag(), |1..=9| 11 | 30 | 33 | 60 | 61) {
        return true;
    }

    #[cfg(target_os = "linux")]
    if matches!(page.name(), "[stack]" | "[heap]") || check_exe(page) || page.name().is_empty() {
        return true;
    }

    false
}

#[cfg(target_os = "macos")]
#[inline]
fn check_exe<Q: VirtualQuery>(map: &Q) -> bool {
    let Some(path) = map.path() else {
        return false;
    };

    if path.starts_with("/usr") {
        return false;
    }

    let mut header = [0; 4];
    File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_or(false, |_| EXE.contains(&header))
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

pub fn default_dump_pages<P, W>(proc: &P, writer: &mut W) -> Result<(), io::Error>
where
    P: ProcessInfo + VirtualMemoryRead,
    W: io::Write,
{
    let pages = proc.get_maps().filter(check_region).collect::<Vec<_>>();
    let pages_w = pages.iter().map(Page::from).collect::<Vec<_>>();

    // let region = pages.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();

    // create_pointer_map_writer(proc, &region, writer)
    Ok(())
}
