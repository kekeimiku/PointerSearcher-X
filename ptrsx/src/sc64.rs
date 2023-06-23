#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::prelude::{FileExt, MetadataExt};
#[cfg(target_os = "windows")]
use std::os::windows::prelude::{FileExt, MetadataExt};
use std::{
    collections::BTreeMap, fs::File, io, marker::PhantomPinned, ops::Bound::Included, path::Path, pin::Pin,
    ptr::NonNull,
};
#[cfg(target_os = "windows")]
trait WindowsFileExt {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> io::Result<()>;
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;
}
#[cfg(target_os = "windows")]
impl WindowsFileExt for File {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        let size = FileExt::seek_read(self, buf, offset)?;
        if size < buf.len() {
            return Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "failed to write whole buffer"));
        }
        Ok(())
    }

    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        FileExt::seek_read(self, buf, offset)
    }
}
#[cfg(target_os = "windows")]
trait WindowsMetadataExt {
    fn size(&self) -> u64;
}
#[cfg(target_os = "windows")]
impl WindowsMetadataExt for std::fs::Metadata {
    fn size(&self) -> u64 {
        MetadataExt::file_size(self)
    }
}

use super::{
    c64::{decode_page_info, Page},
    error::Error,
    s64::{pointer_search, Params},
    PTRHEADER64,
};

#[derive(Default)]
pub struct PtrsxScanner<'a> {
    tmp: Vec<u8>,
    pages: Vec<Page<'a>>,
    map: BTreeMap<usize, usize>,
    _pin: PhantomPinned,
}

impl<'a> PtrsxScanner<'a> {
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    pub fn range_address(&'a self, page: &Page<'a>) -> impl Iterator<Item = usize> + 'a {
        self.map
            .range((Included(page.start as usize), (Included(page.end as _))))
            .map(|(&k, _)| k)
    }

    pub fn get_rev_pointer_map(&self) -> BTreeMap<usize, Vec<usize>> {
        self.map.iter().fold(BTreeMap::new(), |mut acc, (&k, &v)| {
            acc.entry(v).or_default().push(k);
            acc
        })
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Pin<Box<Self>>, Error> {
        unsafe {
            let file = File::open(&path)?;
            let mut buf = [0; 12];
            let mut seek = 0_u64;
            file.read_exact_at(&mut buf, seek)?;
            seek += 12;

            let (header, len) = buf.split_at(8);
            let len = u32::from_le_bytes(*(len.as_ptr() as *const _));

            if !PTRHEADER64.eq(header) {
                return Err("this file is not ptr64".into());
            }

            let mut tmp = vec![0; len as usize];
            file.read_exact_at(&mut tmp, seek)?;
            seek += len as u64;

            let ptrsx = Self { tmp, ..Default::default() };

            let mut boxed = Box::pin(ptrsx);
            let raw_tmp = NonNull::from(&boxed.tmp);
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            let pin = Pin::get_unchecked_mut(mut_ref);
            pin.pages = decode_page_info(raw_tmp.as_ref());

            let size = std::fs::metadata(path)?.size();
            if (size - seek) % 16 != 0 {
                return Err("this file is may be corrupted".into());
            }

            let mut tmp = vec![0; 16 * 1000];
            loop {
                let size = file.read_at(&mut tmp, seek)?;
                if size == 0 {
                    break;
                }
                for b in tmp[..size].chunks(16) {
                    let (addr, content) = b.split_at(8);
                    let addr = usize::from_le_bytes(*(addr.as_ptr() as *const _));
                    let content = usize::from_le_bytes(*(content.as_ptr() as *const _));
                    pin.map.insert(addr, content);
                }
                seek += size as u64;
            }

            Ok(boxed)
        }
    }

    pub fn scan<W: io::Write>(&self, map: &BTreeMap<usize, Vec<usize>>, params: Params<W>) -> io::Result<()> {
        pointer_search(map, params)
    }
}
