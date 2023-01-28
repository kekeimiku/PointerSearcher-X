use std::os::unix::prelude::FileExt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MapRange<'a> {
    pub start: usize,
    pub end: usize,
    pub flags: &'a str,
    pub offset: usize,
    pub dev: &'a str,
    pub inode: usize,
    pub pathname: &'a str,
}

pub struct MapsIter<'a>(core::str::Lines<'a>);

impl<'a> MapsIter<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for MapsIter<'a> {
    type Item = MapRange<'a>;

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
        let pathname = split.next()?.trim_start();

        Some(MapRange { start, end, flags, offset, dev, inode, pathname })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Map {
    pub start: usize,
    pub end: usize,
    pub flags: String,
    pub offset: usize,
    pub dev: String,
    pub inode: usize,
    pub pathname: String,
}

impl From<MapRange<'_>> for Map {
    fn from(value: MapRange<'_>) -> Self {
        Self {
            start: value.start,
            end: value.end,
            flags: value.flags.to_string(),
            offset: value.offset,
            dev: value.dev.to_string(),
            inode: value.inode,
            pathname: value.pathname.to_string(),
        }
    }
}

impl crate::MapExt for Map {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
    fn size(&self) -> usize {
        self.end - self.start
    }
    fn is_exec(&self) -> bool {
        &self.flags[2..3] == "x"
    }
    fn is_read(&self) -> bool {
        &self.flags[0..1] == "r"
    }
    fn is_write(&self) -> bool {
        &self.flags[1..2] == "w"
    }
    fn pathname(&self) -> &str {
        &self.pathname
    }
}

impl crate::MapExt for MapRange<'_> {
    fn start(&self) -> usize {
        self.start
    }
    fn end(&self) -> usize {
        self.end
    }
    fn size(&self) -> usize {
        self.end - self.start
    }
    fn is_exec(&self) -> bool {
        &self.flags[2..3] == "x"
    }
    fn is_read(&self) -> bool {
        &self.flags[0..1] == "r"
    }
    fn is_write(&self) -> bool {
        &self.flags[1..2] == "w"
    }
    fn pathname(&self) -> &str {
        self.pathname
    }
}

pub struct Mem<T>(pub T);

impl<T: FileExt> crate::MemExt for Mem<T> {
    fn read_at(&self, offset: usize, size: usize) -> crate::error::Result<Vec<u8>> {
        let mut buf = vec![0; size];
        self.0.read_at(&mut buf, offset as _)?;
        Ok(buf)
    }

    fn write_at(&self, offset: usize, payload: &[u8]) -> crate::error::Result<usize> {
        Ok(self.0.write_at(payload, offset as _)?)
    }
}

#[cfg(test)]
mod tests {
    use super::{MapRange, MapsIter};
    #[test]
    fn test_linux_parse_proc_maps() {
        let contents: &str = r#"563ea224a000-563ea2259000 r--p 00000000 103:05 5920780 /usr/bin/fish
563ea23ea000-563ea2569000 rw-p 00000000 00:00 0 [heap]
7f9e08000000-7f9e08031000 rw-p 00000000 00:00 0 
563ea224a000-563ea2259000 r--p 00000000 103:05 5920780            /usr/b in/  fish  "#;

        let maps = vec![
            MapRange {
                start: 0x563ea224a000,
                end: 0x563ea2259000,
                offset: 00000000,
                dev: "103:05",
                inode: 5920780,
                flags: "r--p",
                pathname: "/usr/bin/fish",
            },
            MapRange {
                start: 0x563ea23ea000,
                end: 0x563ea2569000,
                offset: 00000000,
                dev: "00:00",
                inode: 0,
                flags: "rw-p",
                pathname: "[heap]",
            },
            MapRange {
                start: 0x7f9e08000000,
                end: 0x7f9e08031000,
                offset: 00000000,
                dev: "00:00",
                inode: 0,
                flags: "rw-p",
                pathname: "",
            },
            MapRange {
                start: 0x563ea224a000,
                end: 0x563ea2259000,
                offset: 00000000,
                dev: "103:05",
                inode: 5920780,
                flags: "r--p",
                pathname: "/usr/b in/  fish  ",
            },
        ];

        let parse_maps = MapsIter::new(contents).collect::<Vec<_>>();

        assert_eq!(parse_maps, maps);
    }
}
