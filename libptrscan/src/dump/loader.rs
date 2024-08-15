use core::{
    mem,
    ops::{Bound, Range},
};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Cursor, Error, Read},
    path::Path,
};

use super::{Header, PointerMap, RangeMap, ARCH64, MAGIC};

pub fn load_pointer_map_file(pathname: impl AsRef<Path>) -> Result<PointerMap, Error> {
    let file = File::open(pathname)?;
    let mut cursor = Cursor::new(file);

    let mut header = [0_u8; mem::size_of::<Header>()];
    cursor.get_mut().read_exact(&mut header)?;
    let header: Header = unsafe { mem::transmute(header) };

    // TODO 检查文件合法性
    if &header.magic != MAGIC || header.arch != ARCH64 {
        return Err(Error::other("invalid pointer_map file"));
    }

    let mut modules = RangeMap::new();
    for _ in 0..header.modules_size {
        let mut buf = [0_u8; mem::size_of::<usize>() * 3];
        cursor.get_mut().read_exact(&mut buf)?;
        let (start, end, size): (usize, usize, usize) = unsafe { mem::transmute(buf) };
        let mut buf = vec![0_u8; size];
        cursor.get_mut().read_exact(&mut buf)?;
        let name = String::from_utf8(buf).unwrap();
        modules.insert(start..end, name);
    }

    const BUF_SIZE: usize = mem::size_of::<usize>() * 0x20000;
    const CHUNK_SIZE: usize = mem::size_of::<usize>() * 2;
    let mut buf = vec![0; BUF_SIZE];

    let mut addr_map = BTreeMap::new();
    loop {
        let size = cursor.get_mut().read(&mut buf)?;
        if size == 0 {
            break;
        }
        for chuks in buf[..size].chunks_exact(CHUNK_SIZE) {
            let (key, value) = chuks.split_at(mem::size_of::<usize>());
            let (key, value) = (
                usize::from_ne_bytes(key.try_into().unwrap()),
                usize::from_ne_bytes(value.try_into().unwrap()),
            );
            addr_map.insert(key, value);
        }
    }

    let points = modules
        .iter()
        .flat_map(|(Range { start, end }, ..)| {
            addr_map.range((Bound::Included(start), Bound::Included(end)))
        })
        .map(|(k, _)| k)
        .copied()
        .collect();

    let mut map: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for (k, v) in addr_map {
        map.entry(v).or_default().push(k)
    }

    Ok(PointerMap { points, map, modules })
}
