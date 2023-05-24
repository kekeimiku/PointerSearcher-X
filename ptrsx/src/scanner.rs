use std::{
    collections::BTreeMap,
    fs::File,
    io,
    ops::Bound::Included,
    os::unix::prelude::{FileExt, MetadataExt},
    path::Path,
};

use utils::consts::{Address, POINTER_SIZE};

use crate::{
    e::PointerSeacher,
    map::{decode_bytes_to_maps, Map},
};

pub struct PtrsXScanner {
    pub pages: Vec<Map>,
    pub bmap: BTreeMap<Address, Address>,
}

impl PtrsXScanner {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let (bmap, map) = load_pointer_map(path)?;
        Ok(Self { pages: map, bmap })
    }

    pub fn pages(&self) -> &[Map] {
        &self.pages
    }

    pub fn range_address<'a>(&'a self, pages: &'a [Map]) -> impl Iterator<Item = Address> + 'a {
        pages
            .iter()
            .flat_map(|Map { start, end, .. }| self.bmap.range((Included(start), Included(end))).map(|(&k, _)| k))
    }

    pub fn default_address(&self) -> impl Iterator<Item = Address> + '_ {
        self.bmap.iter().map(|(&k, _)| k)
    }

    pub fn rev_pointer_map(self) -> BTreeMap<Address, Vec<Address>> {
        self.bmap.into_iter().fold(BTreeMap::new(), |mut acc, (k, v)| {
            acc.entry(v).or_default().push(k);
            acc
        })
    }
}

pub struct PathFindEngine<'a, W> {
    target: Address,
    depth: usize,
    offset: (usize, usize),
    out: &'a mut W,
    startpoints: Vec<Address>,
    engine: PointerSeacher,
}

impl<W> PathFindEngine<'_, W>
where
    W: io::Write,
{
    pub fn find_pointer_path(self) -> io::Result<()> {
        let PathFindEngine { target, depth, offset, out, engine, startpoints } = self;
        let size = depth * 2 + 9;
        out.write_all(&size.to_le_bytes())?;
        engine.path_find_helpers(target, out, offset, depth, size, &startpoints)
    }
}

fn load_pointer_map<P: AsRef<Path>>(path: P) -> io::Result<(BTreeMap<Address, Address>, Vec<Map>)> {
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
