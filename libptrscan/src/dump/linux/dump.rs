use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    mem,
    ops::{Bound, Range},
    os::unix::fs::FileExt,
    path::Path,
};

use super::{Error, Header, PointerMap, RangeMap, RangeSet, ARCH64, MAGIC};

pub fn create_pointer_map(
    mem: &File,
    module_maps: RangeMap<usize, String>,
    unknown_maps: RangeSet<usize>,
) -> Result<PointerMap, Error> {
    let range_maps = unknown_maps
        .into_iter()
        .chain(module_maps.iter().map(|(k, _)| k).cloned())
        .collect::<RangeSet<usize>>();

    let mut addr_map = BTreeMap::new();

    let mut buf = vec![0_u8; 0x100000];
    for Range { start, end } in range_maps.iter() {
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
                .filter(|(_, v)| range_maps.get_range_by_point(v).is_some())
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
    module_maps: RangeMap<usize, String>,
    unknown_maps: RangeSet<usize>,
    path: impl AsRef<Path>,
) -> Result<(), Error> {
    let range_maps = unknown_maps
        .into_iter()
        .chain(module_maps.iter().map(|(k, _)| k).cloned())
        .collect::<RangeSet<usize>>();

    let file = File::options().append(true).create_new(true).open(path)?;
    let mut buffer = BufWriter::with_capacity(0x100000, file);

    buffer.write_all(header(module_maps.len() as u32).as_bytes())?;

    module_maps
        .into_iter()
        .try_for_each(|(Range { start, end }, name)| {
            buffer
                .write_all(&start.to_ne_bytes())
                .and(buffer.write_all(&end.to_ne_bytes()))
                .and(buffer.write_all(&name.len().to_ne_bytes()))
                .and(buffer.write_all(name.as_bytes()))
        })?;

    let mut buf = vec![0_u8; 0x100000];
    for Range { start, end } in range_maps.iter() {
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
                .filter(|(_, v)| range_maps.get_range_by_point(v).is_some())
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
