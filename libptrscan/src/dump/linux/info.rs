use std::{
    fs::{self, File},
    io::{Error, Read},
    path::Path,
};

use super::{RangeMap, RangeSet};

#[allow(dead_code)]
struct Map<'a> {
    start: usize,
    end: usize,
    flags: &'a str,
    offset: usize,
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

    #[allow(dead_code)]
    fn is_exec(&self) -> bool {
        &self.flags[2..3] == "x"
    }

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

const REGIONS: [&str; 3] = ["[anon:libc_malloc]", "[stack]", "[heap]"];
const BSS: &str = "[anon:.bss]";

#[inline]
fn is_a(s: &str) -> bool {
    s.get(0..7).is_some_and(|s| s.eq("/memfd:")) || s.starts_with("/dev/")
}

fn is_elf(s: &str) -> bool {
    let path = Path::new(s);
    if path.is_file() {
        let mut buf = [0; 6];
        File::open(path)
            .and_then(|mut f| f.read_exact(&mut buf))
            .is_ok_and(|_| [0x7f, b'E', b'L', b'F', 2, 1].eq(&buf))
    } else {
        false
    }
}

pub fn list_image_maps(pid: i32) -> Result<RangeMap<usize, String>, Error> {
    let contents = fs::read_to_string(format!("/proc/{pid}/maps"))?;
    let maps = MapIter::new(&contents).collect::<Vec<_>>();

    let mut image_module_maps = RangeMap::new();

    for (a, b) in maps.iter().zip(maps.iter().skip(1)) {
        if a.is_read() && a.is_write() {
            if let Some(module) = a.name().filter(|s| a.inode != 0 && !is_a(s) && is_elf(s)) {
                image_module_maps.insert(a.start()..a.end(), module.to_string());

                if b.name()
                    .is_some_and(|s| s == BSS && b.is_read() && b.is_write())
                {
                    image_module_maps.insert(b.start()..b.end(), format!("{module}:bss"));
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

    for map in maps.filter(|m| m.is_read() && m.is_write()) {
        if map.name().is_some_and(|name| REGIONS.contains(&name)) || map.name().is_none() {
            unknown_maps.insert(map.start()..map.end())
        }
    }

    Ok(unknown_maps)
}
