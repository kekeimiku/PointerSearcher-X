#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::{fs::File, io::Read};
use std::{io, io::Write};

use super::{error::Error, *};

#[cfg(target_os = "linux")]
pub const EXE: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
))]
pub const EXE: [[u8; 4]; 2] = [[0xCA, 0xFE, 0xBA, 0xBE], [0xCF, 0xFA, 0xED, 0xFE]];

#[cfg(target_os = "linux")]
use vmmap::linux::VirtualQueryExt;
#[cfg(target_os = "macos")]
use vmmap::macos::VirtualQueryExt;
#[cfg(target_os = "windows")]
use vmmap::windows::VirtualQueryExt;
use vmmap::{ProcessInfo, VirtualMemoryRead, VirtualQuery};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Page<'a> {
    pub start: usize,
    pub end: usize,
    pub path: &'a str,
}

// fuck sb rust TryFrom https://github.com/rust-lang/rust/issues/50133
pub struct PageTryWrapper<T>(T);

#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86_64")
))]
impl<'a, V> TryFrom<PageTryWrapper<&'a V>> for Page<'a>
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
impl<'a, V> TryFrom<PageTryWrapper<&'a V>> for Page<'a>
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
    let Some(path) = page.path() else {
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
pub fn check_exe<Q: VirtualQueryExt>(page: &Q) -> bool {
    let path = std::path::Path::new(page.name());
    if !path.exists() || path.starts_with("/dev") || path.starts_with("/usr") {
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
    let Some(path) = page.path() else {
        return false;
    };

    if path.starts_with("\\Device\\HarddiskVolume3\\Windows\\System32") {
        return false;
    }

    path.extension().is_some_and(|s| s == "dll" || s == "exe")
}

pub fn default_dump_ptr<P, W>(proc: &P, writer: &mut W) -> Result<(), Error>
where
    P: ProcessInfo + VirtualMemoryRead,
    W: io::Write,
    Error: From<P::Error>,
{
    let pages = proc.get_maps().filter(check_region).collect::<Vec<_>>();
    let region = pages.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();
    let pages_info =
        merge_bases(pages.iter().flat_map(|x| PageTryWrapper(x).try_into())).expect("error: pages is_empty");
    encode_page_info(&pages_info, writer)?;
    create_pointer_map_with_writer(proc, &region, writer)
}

#[inline]
pub fn merge_bases<'a, I>(mut iter: I) -> Option<Vec<Page<'a>>>
where
    I: Iterator<Item = Page<'a>>,
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

fn encode_page_info<W: io::Write>(pages: &[Page<'_>], writer: &mut W) -> io::Result<()> {
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
    #[cfg(target_pointer_width = "64")]
    writer.write_all(&PTRHEADER64)?;
    #[cfg(target_pointer_width = "32")]
    writer.write_all(&PTRHEADER32)?;
    // pages 长度
    writer.write_all(&(tmp.len() as u32).to_le_bytes())?;
    writer.write_all(&tmp)
}

pub fn decode_page_info(bytes: &[u8]) -> Vec<Page<'_>> {
    unsafe {
        let mut i = 0;
        let len = u32::from_le_bytes(*(bytes.as_ptr().cast())) as usize;
        let mut pages = Vec::with_capacity(len);
        i += 4;
        for _ in 0..len {
            let start = usize::from_le_bytes(*(bytes.as_ptr().add(i).cast()));
            i += 8;
            let end = usize::from_le_bytes(*(bytes.as_ptr().add(i).cast()));
            i += 8;
            let len = u32::from_le_bytes(*(bytes.as_ptr().add(i).cast())) as usize;
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
