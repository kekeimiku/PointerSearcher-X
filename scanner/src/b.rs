use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io,
    io::{BufWriter, Write},
    os::unix::prelude::{FileExt, MetadataExt},
    path::Path,
};

use consts::{Address, MAX_BUF_SIZE, POINTER_SIZE};

use crate::map::{Map, MapIter};

pub fn load_pointer_map<P: AsRef<Path>>(path: P) -> io::Result<(BTreeMap<Address, Address>, Vec<Map>)> {
    let file = File::open(path)?;

    let mut seek = 0;
    let mut map = BTreeMap::new();

    let mut buf = [0; 8];
    file.read_exact_at(&mut buf, seek)?;
    seek += buf.len() as u64;
    let size = usize::from_le_bytes(buf);
    let mut buf = vec![0; size];
    file.read_exact_at(&mut buf, seek)?;
    seek += size as u64;
    println!("{}", seek);
    assert_eq!((file.metadata()?.size() - seek) % 16, 0);

    let m = MapIter(String::from_utf8_lossy(&buf).lines()).collect();
    let mut buf = [0; POINTER_SIZE * 100000];
    let chunk_size = POINTER_SIZE * 2;

    loop {
        let size = file.read_at(&mut buf, seek)?;
        if size == 0 {
            break;
        }
        for b in buf.chunks(chunk_size) {
            let (k, v) = b.split_at(POINTER_SIZE);
            let k = Address::from_le_bytes(unsafe { *(k.as_ptr() as *const [u8; 8]) });
            let v = Address::from_le_bytes(unsafe { *(v.as_ptr() as *const [u8; 8]) });
            map.insert(k, v);
        }
        seek += size as u64;
    }

    Ok((map, m))
}

pub fn convert_bin_to_txt<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut buffer = BufWriter::with_capacity(
        MAX_BUF_SIZE,
        OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path.as_ref().with_extension("txt"))?,
    );

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

    let m = MapIter(String::from_utf8_lossy(&mbuf).lines()).collect::<Vec<_>>();
    let mut buf = vec![0; size * 1000];

    loop {
        let n = file.read_at(&mut buf, seek)?;
        if n == 0 {
            break;
        }
        seek += n as u64;

        for bin in buf.chunks(size) {
            let (off, path) = parse_line(bin).ok_or("err").unwrap();
            let ptr = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
            for Map { start, end, path } in m.iter() {
                if (start..end).contains(&&off) {
                    let name = path.file_name().unwrap().to_string_lossy();
                    writeln!(buffer, "{name}+{:#x}->{ptr}", off - start).unwrap();
                }
            }
        }
    }

    Ok(())
}

#[inline(always)]
pub fn parse_line(bin: &[u8]) -> Option<(Address, impl Iterator<Item = i16> + '_)> {
    let line = bin.rsplitn(2, |&n| n == 101).nth(1)?;
    let (off, path) = line.split_at(8);
    let off = Address::from_le_bytes(off.try_into().ok()?);
    let path = path.chunks(2).rev().map(|x| i16::from_le_bytes(x.try_into().unwrap()));

    Some((off, path))
}
