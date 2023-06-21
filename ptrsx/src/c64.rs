#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::{fs::File, io::Read};
use std::{io, io::Write};

#[cfg(target_os = "linux")]
pub const EXE: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const EXE: [[u8; 4]; 2] = [[0xCA, 0xFE, 0xBA, 0xBE], [0xCF, 0xFA, 0xED, 0xFE]];

#[cfg(target_os = "linux")]
use vmmap::vmmap64::VirtualQueryExt;
#[cfg(target_os = "windows")]
use vmmap::vmmap64::VirtualQueryExt;
#[cfg(target_os = "macos")]
use vmmap::vmmap64::VirtualQueryExt;
use vmmap::vmmap64::{ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::{d64::create_pointer_map_writer, PTRHEADER64};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Page<T> {
    pub start: u64,
    pub end: u64,
    pub path: T,
}

// fuck sb rust TryFrom https://github.com/rust-lang/rust/issues/50133
pub struct PageTryWrapper<T>(T);

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl<'a, V> TryFrom<PageTryWrapper<&'a V>> for Page<&'a str>
where
    V: VirtualQuery + VirtualQueryExt,
{
    type Error = ();

    fn try_from(value: PageTryWrapper<&'a V>) -> Result<Self, Self::Error> {
        let path = value.0.path().and_then(|s| s.to_str()).ok_or(())?;
        Ok(Self { start: value.0.start(), end: value.0.end(), path })
    }
}

#[cfg(target_os = "linux")]
impl<'a, V> TryFrom<PageTryWrapper<&'a V>> for Page<&'a str>
where
    V: VirtualQuery + VirtualQueryExt,
{
    type Error = ();

    fn try_from(value: PageTryWrapper<&'a V>) -> Result<Self, Self::Error> {
        let path = value.0.name();
        if !std::path::Path::new(path).has_root() {
            return Err(());
        }
        Ok(Self { start: value.0.start(), end: value.0.end(), path })
    }
}

#[cfg(target_os = "windows")]
impl<'a, V> TryFrom<PageTryWrapper<&'a V>> for Page<&'a str>
where
    V: VirtualQuery + VirtualQueryExt,
{
    type Error = ();

    fn try_from(value: PageTryWrapper<&'a V>) -> Result<Self, Self::Error> {
        let path = value.0.path().and_then(|s| s.to_str()).ok_or(())?;
        if !std::path::Path::new(path).has_root() {
            return Err(());
        }
        Ok(Self { start: value.0.start(), end: value.0.end(), path })
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

    #[cfg(target_os = "windows")]
    check_exe(page)
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

#[cfg(target_os = "windows")]
#[inline]
pub fn check_exe<Q: VirtualQuery + VirtualQueryExt>(page: &Q) -> bool {
    page.path()
        .map_or(false, |f| f.extension().is_some_and(|s| s == "dll" || s == "exe"))
}

pub fn default_dump_ptr<P, W>(proc: &P, writer: &mut W) -> Result<(), io::Error>
where
    P: ProcessInfo + VirtualMemoryRead,
    W: io::Write,
{
    let pages = proc.get_maps().filter(check_region).collect::<Vec<_>>();
    let region = pages.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();
    let pages_info =
        merge_bases(pages.iter().flat_map(|x| PageTryWrapper(x).try_into())).expect("error: pages is_empty");
    encode_page_info(&pages_info, writer)?;
    create_pointer_map_writer(proc, &region, writer)
}

#[inline]
pub fn merge_bases<T, I>(mut iter: I) -> Option<Vec<Page<T>>>
where
    T: PartialEq,
    I: Iterator<Item = Page<T>>,
{
    let mut current = iter.next()?;
    let mut result = Vec::new();
    for page in iter {
        if page.path == current.path {
            current.end = page.end;
        } else {
            result.push(current);
            current = page;
        }
    }
    result.push(current);
    Some(result)
}

fn encode_page_info<W: io::Write>(pages: &[Page<&str>], writer: &mut W) -> io::Result<()> {
    let mut tmp = Vec::new();
    let len = (pages.len() as u32).to_le_bytes();
    tmp.write_all(&len)?;
    for Page { start, end, path } in pages {
        tmp.write_all(&start.to_le_bytes())?;
        tmp.write_all(&end.to_le_bytes())?;
        tmp.write_all(&(path.len() as u32).to_le_bytes())?;
        tmp.write_all(path.as_bytes())?;
    }
    // header 表示这是一个64位ptr文件
    writer.write_all(&PTRHEADER64)?;
    // pages 长度
    writer.write_all(&(tmp.len() as u32).to_le_bytes())?;
    writer.write_all(&tmp)
}

pub fn decode_page_info(bytes: &[u8]) -> Vec<Page<&str>> {
    unsafe {
        let mut i = 0;
        let len = u32::from_le_bytes(*(bytes.as_ptr() as *const _)) as usize;
        let mut pages = Vec::with_capacity(len);
        i += 4;
        for _ in 0..len {
            let start = u64::from_le_bytes(*(bytes.as_ptr().add(i) as *const _));
            i += 8;
            let end = u64::from_le_bytes(*(bytes.as_ptr().add(i) as *const _));
            i += 8;
            let len = u32::from_le_bytes(*(bytes.as_ptr().add(i) as *const _)) as usize;
            i += 4;
            let name = core::str::from_utf8_unchecked(bytes.get_unchecked(i..i + len));
            i += len;
            pages.push(Page { start, end, path: name })
        }

        pages
    }
}

#[test]
fn test_decode_and_encode_map() {
    let pages = vec![
        Page { start: 1, end: 2, path: "value" },
        Page { start: 4, end: 7, path: "va lue" },
    ];
    let mut out = vec![];
    encode_page_info(&pages, &mut out).unwrap();
    assert_eq!(decode_page_info(&out[12..]), pages)
}

#[test]
pub fn test_merge() {
    let pages = vec![
        Page { start: 1, end: 10, path: "value" },
        Page { start: 10, end: 20, path: "value" },
        Page { start: 20, end: 30, path: "nnnn" },
        Page { start: 30, end: 40, path: "hello" },
        Page { start: 40, end: 50, path: "hello" },
        Page { start: 50, end: 60, path: "ddd" },
    ];
    assert_eq!(
        Some(vec![
            Page { start: 1, end: 20, path: "value" },
            Page { start: 20, end: 30, path: "nnnn" },
            Page { start: 30, end: 50, path: "hello" },
            Page { start: 50, end: 60, path: "ddd" }
        ]),
        merge_bases(pages.into_iter())
    )
}
