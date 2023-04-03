use crate::{VirtualMemoryInfo, VirtualMemoryRead, VirtualMemoryWrite};

use super::VirtualQuery;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Map<'a> {
    start: usize,
    end: usize,
    perm: &'a str,
    tag: u8,
    linkname: &'a str,
    pathname: &'a str,
}

impl VirtualQuery for Map<'_> {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn size(&self) -> usize {
        self.end - self.start
    }

    fn is_read(&self) -> bool {
        &self.perm[0..1] == "r"
    }

    fn is_write(&self) -> bool {
        &self.perm[1..2] == "w"
    }

    fn is_exec(&self) -> bool {
        &self.perm[2..3] == "x"
    }

    fn is_stack(&self) -> bool {
        self.tag == 1
    }

    fn is_heap(&self) -> bool {
        self.tag == 2
    }

    fn path(&self) -> &str {
        self.pathname
    }
}

pub struct MapIter<'a>(core::str::Lines<'a>);

impl<'a> MapIter<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for MapIter<'a> {
    type Item = Map<'a>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.0.next()?;
        let mut split = line.splitn(5, ' ');
        let mut range_split = split.next()?.split('-');
        let start = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let end = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let perm = split.next()?;
        let tag = split.next()?.parse().ok()?;
        let linkname = split.next()?.trim_start();
        let pathname = split.next()?.trim_start();

        Some(Map { start, end, perm, tag, linkname, pathname })
    }
}

pub struct Process {
    maps: String,
}

impl VirtualMemoryInfo for Process {
    fn get_maps(&self) -> impl Iterator<Item = impl VirtualQuery + '_> {
        MapIter::new(&self.maps)
    }
}

impl VirtualMemoryRead for Process {
    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> crate::error::Result<usize> {
        todo!()
    }
}

impl VirtualMemoryWrite for Process {
    fn write_at(&self, _offset: usize, _buf: &[u8]) -> crate::error::Result<()> {
        todo!()
    }
}

#[test]
fn test_parse_dump() {
    let contents = r#"102c58000-102c60000 rw- 2 dumpxx/102c58000-102c60000.mem /Users/keke/Code/Github/ups-dev/ups-test/libhello.dylib
102c68000-102c70000 rw- 2 dumpxx/102c68000-102c70000.mem /Users/keke/Code/Github/ups-dev/ups-test/libhello.dylib
102c80000-102c84000 rw- 0 dumpxx/102c80000-102c84000.mem /Users/keke/Code/Github/ups-dev/ups-test/libhello.dylib
126600000-126700000 rw- 2 dumpxx/126600000-126700000.mem 
126800000-127000000 rw- 2 dumpxx/126800000-127000000.mem 
16ca78000-16d270000 rw- 1 dumpxx/16ca78000-16d270000.mem "#;

    let maps = MapIter::new(contents).collect::<Vec<_>>();

    assert_eq!(6, maps.len())
}
