use std::{
    collections::BTreeMap,
    fs::File,
    io,
    os::unix::prelude::{FileExt, MetadataExt},
    path::Path,
};

use utils::consts::{Address, POINTER_SIZE};

use crate::map::{decode_bytes_to_maps, Map};

pub struct PtrsXScanner {
    pub map: Vec<Map>,
    pub bmap: BTreeMap<Address, Address>,
}

impl PtrsXScanner {
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let (bmap, map) = load_pointer_map(path)?;
        Ok(Self { map, bmap })
    }

    pub fn map(&self) -> &[Map] {
        &self.map
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
