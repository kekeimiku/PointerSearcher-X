use std::{
    fs::{self, File}, io::{self, BufRead, BufReader, Cursor},
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
};

use super::{Error, Pid, ProcessInfo, Result, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery, VirtualQueryExt};

pub struct Process {
    pub pid: Pid,
    pub pathname: PathBuf,
    pub mapping: File,
    pub memory: File,
}

impl VirtualMemoryRead for Process {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> Result<usize> {
        self.memory.read_at(buf, offset as u64).map_err(Error::ReadMemory)
    }

    fn read_exact_at(&self, buf: &mut [u8], offset: usize) -> Result<()> {
        self.memory.read_exact_at(buf, offset as u64).map_err(Error::ReadMemory)
    }
}

impl VirtualMemoryWrite for Process {
    fn write_at(&self, buf: &[u8], offset: usize) -> Result<usize> {
        self.memory.write_at(buf, offset as u64).map_err(Error::WriteMemory)
    }

    fn write_all_at(&self, buf: &[u8], offset: usize) -> Result<()> {
        self.memory.write_all_at(buf, offset as u64).map_err(Error::WriteMemory)
    }
}

impl ProcessInfo for Process {
    fn pid(&self) -> Pid {
        self.pid
    }

    fn app_path(&self) -> &Path {
        &self.pathname
    }

    fn get_maps(&self) -> impl Iterator<Item = Result<Mapping>> {
        Iter::new(BufReader::new(&self.mapping)).map(|x| x.map_err(Error::QueryMapping))
    }
}

impl Process {
    pub fn open(pid: Pid) -> Result<Self> {
        Self::_open(pid).map_err(Error::OpenProcess)
    }

    fn _open(pid: Pid) -> Result<Self, io::Error> {
        let mapping = File::open(format!("/proc/{pid}/maps"))?;
        let pathname = fs::read_link(format!("/proc/{pid}/exe"))?;
        let handle = File::options()
            .read(true)
            .write(true)
            .open(format!("/proc/{pid}/mem"))?;
        Ok(Self { pid, pathname, mapping, memory: handle })
    }
}

#[allow(dead_code)]
pub struct Mapping {
    pub start: usize,
    pub end: usize,
    pub flags: String,
    pub offset: usize,
    pub dev: String,
    pub inode: usize,
    pub name: Option<String>,
}

impl VirtualQuery for Mapping {
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

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl VirtualQueryExt for Mapping {
    fn offset(&self) -> usize {
        self.offset
    }

    fn dev(&self) -> &str {
        &self.dev
    }

    fn inode(&self) -> usize {
        self.inode
    }
}

pub struct Iter<R> {
    buffer: String,
    cursor: Cursor<R>,
}

impl<R> Iter<R> {
    fn new(r: R) -> Self {
        Self { buffer: String::with_capacity(0x100), cursor: Cursor::new(r) }
    }
}

impl<R: BufRead> Iterator for Iter<R> {
    type Item = Result<Mapping, io::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let size = match self.cursor.get_mut().read_line(&mut self.buffer) {
            Ok(0) => return None,
            Ok(size) => size,
            Err(err) => return Some(Err(err)),
        };
        let mut split = self.buffer[..size].trim_end().splitn(6, ' ');
        let mut range_split = split.next()?.split('-');
        let start = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let end = usize::from_str_radix(range_split.next()?, 16).ok()?;
        let flags = split.next()?.to_string();
        let offset = usize::from_str_radix(split.next()?, 16).ok()?;
        let dev = split.next()?.to_string();
        let inode = split.next()?.parse().ok()?;
        let name = split.next().map(|s| s.trim_start().to_string());
        self.buffer.clear();
        Some(Ok(Mapping { start, end, flags, offset, dev, inode, name }))
    }
}
