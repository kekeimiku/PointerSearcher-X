use core::{
    mem,
    ops::{Bound, Range},
};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use machx::{
    kern_return::KERN_SUCCESS, port::mach_port_name_t, vm::mach_vm_read_overwrite,
    vm_types::mach_vm_address_t,
};

use super::{Error, Header, PointerMap, RangeMap, RangeSet, ARCH64, MAGIC};

// 苹果的 mach_vm_read_overwrite 仅有精准模式，所以手动计算它
struct ChunkIter {
    max: usize,
    size: usize,
    pos: usize,
}

impl ChunkIter {
    #[inline]
    fn new(max: usize, size: usize) -> Self {
        Self { max, size, pos: 0 }
    }
}

impl Iterator for ChunkIter {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.max {
            None
        } else {
            let curr = self.pos;
            self.pos = (self.pos + self.size).min(self.max);
            Some((curr, self.pos - curr))
        }
    }
}

// TODO: 可以轻松的转为并行，但不是现在
pub fn create_pointer_map(
    task: mach_port_name_t,
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
        for (off, size) in ChunkIter::new(size, 0x100000) {
            let buf = &mut buf[..size];
            let mut outsize = 0;
            let kr = unsafe {
                mach_vm_read_overwrite(
                    task,
                    (start + off) as u64,
                    size as u64,
                    buf.as_mut_ptr() as mach_vm_address_t,
                    &mut outsize,
                )
            };
            if kr != KERN_SUCCESS || outsize as usize != size {
                break;
            }

            for (k, v) in buf
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

    let points = module_maps
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
    task: mach_port_name_t,
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
        for (off, size) in ChunkIter::new(size, 0x100000) {
            let buf = &mut buf[..size];
            let mut outsize = 0;
            let kr = unsafe {
                mach_vm_read_overwrite(
                    task,
                    (start + off) as u64,
                    size as u64,
                    buf.as_mut_ptr() as mach_vm_address_t,
                    &mut outsize,
                )
            };
            if kr != KERN_SUCCESS || outsize as usize != size {
                break;
            }

            for (k, v) in buf
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
