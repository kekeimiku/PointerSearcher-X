#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::prelude::{FileExt, MetadataExt};
#[cfg(target_os = "windows")]
use std::os::windows::prelude::{FileExt, MetadataExt};
use std::{collections::BTreeMap, fs::File, io, ops::Bound::Included, path::Path};
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

use self_cell::self_cell;

use super::*;

const PTRSIZE: usize = std::mem::size_of::<usize>();

type Dependent<'a> = Vec<Page<'a>>;

pub struct Owner {
    vec: Vec<u8>,
    map: BTreeMap<usize, usize>,
}

self_cell!(
    pub struct PtrsxScanner<'a> {
        owner: Owner,

        #[covariant]
        dependent: Dependent,
    }
);

impl<'a> PtrsxScanner<'a> {
    pub fn pages(&self) -> &[Page] {
        self.borrow_dependent()
    }

    pub fn range_address(&'a self, page: &Page<'a>) -> impl Iterator<Item = usize> + 'a {
        self.borrow_owner()
            .map
            .range((Included(page.start), (Included(page.end))))
            .map(|(&k, _)| k)
    }

    pub fn get_rev_pointer_map(&self) -> BTreeMap<usize, Vec<usize>> {
        self.borrow_owner()
            .map
            .iter()
            .fold(BTreeMap::new(), |mut acc, (&k, &v)| {
                acc.entry(v).or_default().push(k);
                acc
            })
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        unsafe {
            let file = File::open(&path)?;
            let mut buf = [0; 12];
            let mut seek = 0_u64;
            file.read_exact_at(&mut buf, seek)?;
            seek += 12;

            let (header, len) = buf.split_at(8);
            let len = u32::from_le_bytes(*(len.as_ptr().cast()));

            #[cfg(target_pointer_width = "64")]
            if !PTRHEADER64.eq(header) {
                return Err("this file is not ptr64".into());
            }
            #[cfg(target_pointer_width = "32")]
            if !PTRHEADER32.eq(header) {
                return Err("this file is not ptr32".into());
            }

            let mut vec = vec![0; len as usize];
            file.read_exact_at(&mut vec, seek)?;
            seek += len as u64;

            let size = std::fs::metadata(path)?.size();
            if (size - seek) % (PTRSIZE * 2) as u64 != 0 {
                return Err("this file is may be corrupted".into());
            }

            let mut map = BTreeMap::new();
            let mut tmp = vec![0; (PTRSIZE * 2) * 1000];
            loop {
                let size = file.read_at(&mut tmp, seek)?;
                if size == 0 {
                    break;
                }
                for b in tmp[..size].chunks(PTRSIZE * 2) {
                    let (addr, content) = b.split_at(PTRSIZE);
                    let addr = usize::from_le_bytes(*(addr.as_ptr().cast()));
                    let content = usize::from_le_bytes(*(content.as_ptr().cast()));
                    map.insert(addr, content);
                }
                seek += size as u64;
            }

            Ok(Self::new(Owner { vec, map }, |x| decode_page_info(&x.vec)))
        }
    }

    pub fn scan<W: io::Write>(&self, map: &BTreeMap<usize, Vec<usize>>, params: Params<W>) -> io::Result<()> {
        pointer_chain_scanner(map, params)
    }
}
