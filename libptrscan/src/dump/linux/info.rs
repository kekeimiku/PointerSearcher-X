use std::{
    fs::{self, File},
    io::{Error, Read},
    path::Path,
};

use super::{RangeMap, RangeSet};

struct Map<'a> {
    start: usize,
    end: usize,
    flags: &'a str,
    #[allow(dead_code)]
    offset: usize,
    #[allow(dead_code)]
    dev: &'a str,
    inode: usize,
    name: Option<&'a str>,
}

impl Map<'_> {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn is_read(&self) -> bool {
        &self.flags[0..1] == "r"
    }

    fn is_write(&self) -> bool {
        &self.flags[1..2] == "w"
    }

    // fn is_exec(&self) -> bool {
    //     &self.flags[2..3] == "x"
    // }

    fn name(&self) -> Option<&str> {
        self.name
    }
}

struct MapIter<'a>(core::str::Lines<'a>);

impl<'a> MapIter<'a> {
    fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for MapIter<'a> {
    type Item = Map<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.0.next()?;
        let mut split = line.splitn(6, ' ');
        let mut range_split = split.next()?.split('-');
        let start = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let end = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let flags = split.next()?;
        let offset = usize::from_str_radix(split.next()?, 16).ok()?;
        let dev = split.next()?;
        let inode = split.next()?.parse().ok()?;
        let name = split.next()?.trim_start();
        let name = if name.is_empty() { None } else { Some(name) };

        Some(Map { start, end, flags, offset, dev, inode, name })
    }
}

pub fn list_image_maps(pid: i32) -> Result<RangeMap<usize, String>, Error> {
    let contents = fs::read_to_string(format!("/proc/{pid}/maps"))?;
    let maps = MapIter::new(&contents);

    let mut image_module_maps = RangeMap::new();
    let mut buf = [0; 8];

    for map in maps.filter(|m| m.is_read() && m.is_write()) {
        if let Some(name) = map.name() {
            if map.inode != 0 {
                if !name.get(0..7).is_some_and(|s| s.eq("/memfd:")) && !name.starts_with("/dev/") {
                    let path = Path::new(name);
                    if path.is_file() {
                        // TODO 判断文件是否是 elf64 小端
                        let is_elf = File::open(path)
                            .and_then(|mut f| f.read_exact(&mut buf))
                            .is_ok_and(|_| [0x7f, b'E', b'L', b'F', 2, 1].eq(&buf[0..6]));
                        if is_elf {
                            image_module_maps.insert(map.start()..map.end(), name.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(image_module_maps)
}

pub fn list_unknown_maps(pid: i32) -> Result<RangeSet<usize>, Error> {
    let contents = fs::read_to_string(format!("/proc/{pid}/maps"))?;
    let maps = MapIter::new(&contents);

    let mut unknown_maps = RangeSet::new();

    const REGIONS: [&str; 4] = ["[anon:.bss]", "[anon:libc_malloc]", "[stack]", "[heap]"];

    for map in maps.filter(|m| m.is_read() && m.is_write()) {
        if map.name().is_some_and(|name| REGIONS.contains(&name)) || map.name().is_none() {
            unknown_maps.insert(map.start()..map.end())
        }
    }

    Ok(unknown_maps)
}
