use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, BufWriter, Write},
    ops::Bound::Included,
    os::unix::prelude::{FileExt, MetadataExt, OsStrExt},
    path::Path,
};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use super::{
    consts::{Address, POINTER_SIZE},
    map::{decode_bytes_to_maps, Page},
};
use crate::{consts::MODEL1, e::PointerSeacher};

pub struct PtrsXScanner {
    pub pages: Vec<Page>,
    pub bmap: BTreeMap<Address, Address>,
}

impl PtrsXScanner {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let (bmap, map) = load_pointer_map(path)?;
        Ok(Self { pages: map, bmap })
    }

    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    pub fn range_address<'a>(&'a self, page: &'a Page) -> impl Iterator<Item = Address> + 'a {
        self.bmap
            .range((Included(page.start), (Included(page.end))))
            .map(|(&k, _)| k)
    }

    pub fn flat_range_address<'a>(&'a self, pages: &'a [Page]) -> impl Iterator<Item = Address> + 'a {
        pages.iter().flat_map(|page| self.range_address(page))
    }

    pub fn default_address(&self) -> impl Iterator<Item = Address> + '_ {
        self.bmap.iter().map(|(&k, _)| k)
    }

    pub fn rev_pointer_map(&self) -> BTreeMap<Address, Vec<Address>> {
        self.bmap.iter().fold(BTreeMap::new(), |mut acc, (&k, &v)| {
            acc.entry(v).or_default().push(k);
            acc
        })
    }
}

fn load_pointer_map<P: AsRef<Path>>(path: P) -> io::Result<(BTreeMap<Address, Address>, Vec<Page>)> {
    let file = File::open(path)?;

    let mut seek = 0;
    let mut buf = [0; 8];
    file.read_exact_at(&mut buf, seek)?;
    seek += buf.len() as u64;
    let size = usize::from_le_bytes(buf);
    let mut buf = vec![0; size];
    file.read_exact_at(&mut buf, seek)?;
    seek += size as u64;
    assert_eq!((file.metadata()?.size() - seek) % 16, 0);

    let mut map = BTreeMap::new();

    let m = decode_bytes_to_maps(&buf);
    let mut buf = [0; POINTER_SIZE * 100000];
    let chunk_size = POINTER_SIZE * 2;

    loop {
        let size = file.read_at(&mut buf, seek)?;
        if size == 0 {
            break;
        }
        for b in buf.chunks(chunk_size) {
            let (addr, content) = b.split_at(POINTER_SIZE);
            let addr = Address::from_le_bytes(unsafe { *(addr.as_ptr() as *const [u8; 8]) });
            let content = Address::from_le_bytes(unsafe { *(content.as_ptr() as *const [u8; 8]) });
            map.insert(addr, content);
        }
        seek += size as u64;
    }

    Ok((map, m))
}

pub struct ScannerParm {
    pub target: Address,
    pub depth: usize,
    pub offset: (usize, usize),
    pub pages: Vec<Page>,
}

impl PtrsXScanner {
    pub fn scanner(&self, parms: ScannerParm) -> Result<(), std::io::Error> {
        let ScannerParm { ref target, depth, offset, pages } = parms;
        let pointer_map = self.rev_pointer_map();

        pages
            .par_iter()
            .map(|page| (page, self.range_address(page).collect::<Vec<_>>()))
            .try_for_each(|(Page { start, path, .. }, startpoints)| {
                let name = path.file_name().and_then(|f| f.to_str()).unwrap();
                let path = path.as_os_str().as_bytes();
                let file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create_new(true)
                    .open(format!("{target:x}-{name}.scandata"))?;
                let mut writer = BufWriter::new(file);
                let size = depth * 2 + 9;
                writer.write_all(&MODEL1)?;
                writer.write_all(&size.to_le_bytes())?;
                writer.write_all(&path.len().to_le_bytes())?;
                writer.write_all(path)?;
                let pointer_search = PointerSeacher(&pointer_map);
                pointer_search.path_find_helpers(*target, *start, &mut writer, offset, depth, size, &startpoints)
            })?;

        Ok(())
    }
}
