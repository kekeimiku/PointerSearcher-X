use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    mem,
    ops::{Bound, Range},
    os::unix::fs::FileExt,
    path::Path,
};

use super::{Error, Header, ModuleMap, PointerMap, ARCH64, MAGIC};

#[inline]
fn is_pointer(range_maps: &[Range<usize>], addr: &usize) -> bool {
    range_maps
        .binary_search_by(|Range { start, end }| match (start..end).contains(&addr) {
            true => Ordering::Equal,
            false => start.cmp(addr),
        })
        .is_ok()
}

pub fn create_pointer_map(
    mem: &File,
    module_maps: ModuleMap<usize, String>,
    unknown_maps: &[Range<usize>],
) -> Result<PointerMap, Error> {
    let mut range_maps = Vec::with_capacity(128);
    range_maps.extend(module_maps.iter().map(|(range, _)| range.clone()));
    range_maps.extend_from_slice(unknown_maps);
    range_maps.sort_unstable_by_key(|r| r.start);
    range_maps.dedup();

    let mut addr_map = BTreeMap::new();

    let mut buf = vec![0_u8; 0x100000];
    for Range { start, end } in &range_maps {
        let (start, size) = (start, end - start);
        for off in (0..size).step_by(0x100000) {
            let Ok(size) = mem.read_at(&mut buf, (start + off) as u64) else {
                break;
            };

            for (k, v) in buf[..size]
                .windows(mem::size_of::<usize>())
                .enumerate()
                .step_by(mem::size_of::<usize>())
                .map(|(k, v)| (k, usize::from_ne_bytes(v.try_into().unwrap())))
                .filter(|(_, v)| is_pointer(&range_maps, v))
            {
                let k = start + off + k;
                addr_map.insert(k, v);
            }
        }
    }

    let points: Vec<_> = module_maps
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

    Ok(PointerMap { points, map, modules: module_maps })
}

fn header(size: u32) -> Header {
    Header {
        magic: *MAGIC,
        arch: ARCH64,
        _r: [0; 116],
        modules_size: size,
    }
}

pub fn create_pointer_map_file(
    mem: &File,
    module_maps: &ModuleMap<usize, String>,
    unknown_maps: &[Range<usize>],
    path: impl AsRef<Path>,
) -> Result<(), Error> {
    let mut range_maps = Vec::with_capacity(128);
    range_maps.extend(module_maps.iter().map(|(range, _)| range.clone()));
    range_maps.extend_from_slice(unknown_maps);
    // TODO: 正常情况下根本不需要排序以及删除重复元素
    range_maps.sort_unstable_by_key(|r| r.start);
    range_maps.dedup();

    let file = File::options().append(true).create_new(true).open(path)?;
    let mut buffer = BufWriter::with_capacity(0x100000, file);

    buffer.write_all(header(module_maps.len() as u32).as_bytes())?;

    module_maps
        .iter()
        .try_for_each(|(Range { start, end }, name)| {
            buffer
                .write_all(&start.to_ne_bytes())
                .and(buffer.write_all(&end.to_ne_bytes()))
                .and(buffer.write_all(&name.len().to_ne_bytes()))
                .and(buffer.write_all(name.as_bytes()))
        })?;

    let mut buf = vec![0_u8; 0x100000];
    for Range { start, end } in &range_maps {
        let (start, size) = (start, end - start);
        for off in (0..size).step_by(0x100000) {
            let Ok(size) = mem.read_at(&mut buf, (start + off) as u64) else {
                break;
            };

            for (k, v) in buf[..size]
                .windows(mem::size_of::<usize>())
                .enumerate()
                .step_by(mem::size_of::<usize>())
                .map(|(k, v)| (k, usize::from_ne_bytes(v.try_into().unwrap())))
                .filter(|(_, v)| is_pointer(&range_maps, v))
            {
                let k = start + off + k;
                buffer
                    .write_all(&k.to_ne_bytes())
                    .and(buffer.write_all(&v.to_ne_bytes()))?;
            }
        }
    }

    Ok(())
}
