use core::{
    cmp::Ordering,
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

use super::{Error, Header, ModuleMap, PointerMap, ARCH64, MAGIC};

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

// 仅仅是猜测是否为指针
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
    task: mach_port_name_t,
    module_maps: ModuleMap<usize, String>,
    unknown_maps: &[Range<usize>],
) -> Result<PointerMap, Error> {
    let mut range_maps = Vec::with_capacity(128);
    range_maps.extend(module_maps.iter().map(|(range, _)| range.clone()));
    range_maps.extend_from_slice(unknown_maps);
    // TODO: 正常情况下根本不需要排序以及删除重复元素
    range_maps.sort_unstable_by_key(|r| r.start);
    range_maps.dedup();

    let mut addr_map = BTreeMap::new();

    let mut buf = vec![0_u8; 0x100000];
    for Range { start, end } in &range_maps {
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
    task: mach_port_name_t,
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
