use std::{
    collections::BTreeMap,
    fs::File,
    io,
    ops::Bound::Included,
    os::unix::prelude::{FileExt, MetadataExt},
    path::Path,
};

use super::{
    c64::{decode_page_info, Page},
    error::Error,
    s64::{pointer_search, Params},
    PTRHEADER64,
};

#[derive(Default)]
pub struct PtrsxScanner {
    pages: Vec<u8>,
    map: BTreeMap<usize, usize>,
}

impl PtrsxScanner {
    pub fn pages(&self) -> Vec<Page<&str>> {
        decode_page_info(&self.pages)
    }

    pub fn range_address<'a, T>(&'a self, page: &'a Page<T>) -> impl Iterator<Item = usize> + 'a {
        self.map
            .range((Included(page.start as usize), (Included(page.end as _))))
            .map(|(&k, _)| k)
    }

    pub fn flat_range_address<'a, T>(&'a self, pages: &'a [Page<T>]) -> impl Iterator<Item = usize> + 'a {
        pages.iter().flat_map(|page| self.range_address(page))
    }

    pub fn get_rev_pointer_map(&self) -> BTreeMap<usize, Vec<usize>> {
        self.map.iter().fold(BTreeMap::new(), |mut acc, (&k, &v)| {
            acc.entry(v).or_default().push(k);
            acc
        })
    }

    pub fn load_pointer_map_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
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

            let mut buf = vec![0; len as usize];
            file.read_exact_at(&mut buf, seek)?;
            seek += len as u64;
            self.pages = buf;

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
                    self.map.insert(addr, content);
                }
                seek += size as u64;
            }

            Ok(())
        }
    }

    pub fn scan<W: io::Write>(&self, map: &BTreeMap<usize, Vec<usize>>, params: Params<W>) -> io::Result<()> {
        pointer_search(map, params)
    }
}
