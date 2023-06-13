use std::{
    collections::BTreeMap,
    fs::File,
    ops::Bound::Included,
    os::unix::prelude::{FileExt, MetadataExt},
    path::Path,
};

use super::{
    c64::{decode_page_info, Page},
    error::Error,
    PTRHEADER64,
};

#[derive(Default)]
pub struct PtrsxScanner {
    pages: Vec<u8>,
    pub map: BTreeMap<u64, u64>,
}

impl PtrsxScanner {
    pub fn pages(&self) -> Vec<Page<&str>> {
        decode_page_info(&self.pages)
    }

    pub fn get_range_address<'b, T>(&'b self, pages: &'b [Page<T>]) -> impl Iterator<Item = u64> + 'b {
        pages
            .iter()
            .flat_map(|Page { start, end, .. }| self.map.range((Included(start), Included(end))).map(|(&k, _)| k))
    }

    pub fn into_rev_pointer_map(self) -> BTreeMap<u64, Vec<u64>> {
        self.map.into_iter().fold(BTreeMap::new(), |mut acc, (k, v)| {
            acc.entry(v).or_default().push(k);
            acc
        })
    }

    pub fn load_pointer_map<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
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
                for b in tmp.chunks(16) {
                    let (addr, content) = b.split_at(8);
                    let addr = u64::from_le_bytes(*(addr.as_ptr() as *const _));
                    let content = u64::from_le_bytes(*(content.as_ptr() as *const _));
                    self.map.insert(addr, content);
                }
                seek += size as u64;
            }

            Ok(())
        }
    }
}
