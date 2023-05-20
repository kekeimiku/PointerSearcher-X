use std::{collections::BTreeMap, fs::File, io, path::Path};

use dumper::map::{decode_bytes_to_maps, Map};
use utils::{
    consts::{Address, POINTER_SIZE},
    file::{FileExt, MetadataExt},
};

pub fn load_pointer_map<P: AsRef<Path>>(path: P) -> io::Result<(BTreeMap<Address, Address>, Vec<Map>)> {
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
            let (start_addr, end_addr) = b.split_at(POINTER_SIZE);
            let start = Address::from_le_bytes(unsafe { *(start_addr.as_ptr() as *const [u8; 8]) });
            let end = Address::from_le_bytes(unsafe { *(end_addr.as_ptr() as *const [u8; 8]) });
            map.insert(start, end);
        }
        seek += size as u64;
    }

    Ok((map, m))
}

pub fn convert_bin_to_txt<P: AsRef<Path>, W: io::Write>(path: P, mut out: W) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut seek = 0;

    let mut buf = [0; 8];
    file.read_exact_at(&mut buf, seek)?;
    seek += buf.len() as u64;
    let size = usize::from_le_bytes(buf);
    let mut mbuf = vec![0; size];
    file.read_exact_at(&mut mbuf, seek)?;
    seek += size as u64;
    file.read_exact_at(&mut buf, seek)?;
    seek += buf.len() as u64;

    let size = usize::from_le_bytes(buf);

    assert_eq!((file.metadata()?.size() - seek) % size as u64, 0);

    let m = decode_bytes_to_maps(&mbuf);
    let mut buf = vec![0; size * 1000];

    loop {
        let n = file.read_at(&mut buf, seek)?;
        if n == 0 {
            break;
        }
        seek += n as u64;

        for bin in buf[..n].chunks(size) {
            let (off, path) = wrap_parse_line(bin)?;
            let ptr = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
            for Map { start, end, path } in m.iter() {
                if (start..end).contains(&&off) {
                    let name = path.file_name().and_then(|f| f.to_str()).ok_or("get file name error")?;
                    writeln!(out, "{name}+{:#x}->{ptr}", off - start)?;
                }
            }
        }
    }

    Ok(())
}

#[inline(always)]
pub fn wrap_parse_line(bin: &[u8]) -> Result<(Address, impl Iterator<Item = i16> + '_), &'static str> {
    parse_line(bin).ok_or("parse error")
}

#[inline(always)]
fn parse_line(bin: &[u8]) -> Option<(Address, impl Iterator<Item = i16> + '_)> {
    let line = bin.rsplitn(2, |&n| n == 101).nth(1)?;
    let (off, path) = line.split_at(8);
    let off = Address::from_le_bytes(off.try_into().unwrap());
    let path = path.chunks(2).rev().map(|x| i16::from_le_bytes(x.try_into().unwrap()));

    Some((off, path))
}
