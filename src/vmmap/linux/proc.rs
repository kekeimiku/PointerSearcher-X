use std::{
    fmt::Display,
    fs::{self, File},
    os::unix::prelude::FileExt,
    sync::Arc,
};

use super::{Error, Result, VirtualMemoryInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Map<'a> {
    start: usize,
    end: usize,
    flags: &'a str,
    offset: usize,
    dev: &'a str,
    inode: usize,
    pathname: &'a str,
}

impl Display for Map<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
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
        &self.flags[0..1] == "r"
    }

    fn is_write(&self) -> bool {
        &self.flags[1..2] == "w"
    }

    fn is_exec(&self) -> bool {
        &self.flags[2..3] == "x"
    }

    fn is_stack(&self) -> bool {
        self.pathname == "[stack]"
    }

    fn is_heap(&self) -> bool {
        self.pathname == "[heap]"
    }

    fn path(&self) -> &str {
        if self.is_stack() || self.is_heap() {
            ""
        } else {
            self.pathname
        }
    }

    fn name(&self) -> &str {
        let name = &self.path().rsplit_once('/').map(|s| s.1).unwrap_or_default();
        if name.len() > 16 {
            &name[..16]
        } else {
            name
        }
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
        let mut split = line.splitn(6, ' ');
        let mut range_split = split.next()?.split('-');
        let start = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let end = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let flags = split.next()?;
        let offset = usize::from_str_radix(split.next()?, 16).ok()?;
        let dev = split.next()?;
        let inode = split.next()?.parse::<usize>().ok()?;
        let pathname = split.next()?.trim_start();

        Some(Map { start, end, flags, offset, dev, inode, pathname })
    }
}

#[derive(Clone)]
pub struct Process<T> {
    pub pid: i32,
    pathname: String,
    maps: String,
    handle: T,
}

impl Process<Arc<File>> {
    pub fn open(pid: i32) -> Result<Self> {
        Self::o(pid).map_err(Error::OpenProcess)
    }

    fn o(pid: i32) -> Result<Self, std::io::Error> {
        let maps = fs::read_to_string(format!("/proc/{pid}/maps"))?;
        let pathname = fs::read_link(format!("/proc/{pid}/exe"))?
            .to_string_lossy()
            .to_string();
        let handle = Arc::new(File::open(format!("/proc/{pid}/mem"))?);

        Ok(Self { pid, pathname, maps, handle })
    }

    pub fn pathname(&self) -> &str {
        &self.pathname
    }
}

impl VirtualMemoryRead for Process<Arc<File>> {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        self.handle.read_at(buf, offset as _).map_err(Error::ReadMemory)
    }
}

impl VirtualMemoryWrite for Process<Arc<File>> {
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<()> {
        self.handle
            .write_at(buf, offset as _)
            .map(drop)
            .map_err(Error::WriteMemory)
    }
}

impl VirtualMemoryInfo for Process<Arc<File>> {
    fn get_maps(&self) -> impl Iterator<Item = impl VirtualQuery + Clone + Display + '_> {
        MapIter::new(&self.maps)
    }
}
