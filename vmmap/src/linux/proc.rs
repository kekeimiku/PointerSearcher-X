use std::{
    fs,
    fs::File,
    io,
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
};

use super::{Error, Pid, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery, VirtualQueryExt};

pub struct Process {
    pub pid: Pid,
    pathname: PathBuf,
    maps: String,
    handle: File,
}

impl VirtualMemoryRead for Process {
    type Error = Error;

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.handle.read_at(buf, offset).map_err(Error::ReadMemory)
    }
}

impl VirtualMemoryWrite for Process {
    type Error = Error;

    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<(), Self::Error> {
        self.handle.write_at(buf, offset).map(drop).map_err(Error::WriteMemory)
    }
}

impl ProcessInfo for Process {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn app_path(&self) -> &Path {
        &self.pathname
    }

    fn get_maps(&self) -> impl Iterator<Item = Page> + '_ {
        PageIter::new(&self.maps)
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self, Error> {
        Self::o(pid).map_err(Error::OpenProcess)
    }

    fn o(pid: Pid) -> Result<Self, io::Error> {
        let maps = fs::read_to_string(format!("/proc/{pid}/maps"))?;
        let pathname = fs::read_link(format!("/proc/{pid}/exe"))?;
        let handle = File::open(format!("/proc/{pid}/mem"))?;
        Ok(Self { pid, pathname, maps, handle })
    }
}

#[allow(dead_code)]
pub struct Page<'a> {
    start: usize,
    end: usize,
    flags: &'a str,
    offset: usize,
    dev: &'a str,
    inode: usize,
    pathname: &'a str,
}

impl VirtualQuery for Page<'_> {
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

    fn path(&self) -> Option<&Path> {
        let path = Path::new(&self.pathname);
        path.exists().then_some(path)
    }
}

impl VirtualQueryExt for Page<'_> {
    fn name(&self) -> &str {
        self.pathname
    }
}

pub struct PageIter<'a>(core::str::Lines<'a>);

impl<'a> PageIter<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for PageIter<'a> {
    type Item = Page<'a>;

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

        Some(Page { start, end, flags, offset, dev, inode, pathname })
    }
}
