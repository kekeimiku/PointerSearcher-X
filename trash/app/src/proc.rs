use std::fs::{read_dir, read_to_string, File, OpenOptions, ReadDir};

use crate::error::Result;

pub trait MemExt {
    fn read_at(&self, addr: usize, size: usize) -> Result<Vec<u8>>;
    fn write_at(&self, addr: usize, payload: &[u8]) -> Result<usize>;
}

pub trait MapsExt {
    fn size(&self) -> usize;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn is_read(&self) -> bool;
    fn is_write(&self) -> bool;
    fn is_exec(&self) -> bool;
    fn pathname(&self) -> &str;
}

pub trait ProcessExt {
    fn pid(&self) -> i32;
    fn name(&self) -> &str;
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct Maps<'a> {
    start: usize,
    end: usize,
    flags: &'a str,
    offset: usize,
    dev: &'a str,
    inode: usize,
    pathname: &'a str,
}

impl MapsExt for Maps<'_> {
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

pub struct MapsIter<'a>(core::str::Lines<'a>);

impl<'a> MapsIter<'a> {
    pub fn new(contents: &'a str) -> Self {
        Self(contents.lines())
    }
}

impl<'a> Iterator for MapsIter<'a> {
    type Item = Maps<'a>;
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

        Some(Maps { start, end, flags, offset, dev, inode, pathname })
    }
}

pub fn find_region<T: MapsExt>(addr: usize, maps: &[T]) -> Option<&T> {
    maps.iter().find(|m| m.start() < addr && m.end() > addr)
}

pub struct Process {
    pid: i32,
    name: String,
}

impl ProcessExt for Process {
    fn pid(&self) -> i32 {
        self.pid
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct ProcessIter(ReadDir);

impl ProcessIter {
    pub fn new() -> Result<Self> {
        Ok(Self(read_dir("/proc")?))
    }
}

impl Iterator for ProcessIter {
    type Item = Process;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let dir = self.0.next()?.ok()?;
        let path = dir.path().join("comm");
        if path.is_file() {
            if let Ok(pid) = dir.file_name().to_str()?.parse::<i32>() {
                let name = read_to_string(path).ok()?.trim_end().to_string();
                return Some(Process { pid, name });
            }
        }
        self.next()
    }
}

pub fn get_thread_list_by_pid(pid: i32) -> Result<Vec<i32>> {
    Ok(read_dir(format!("/proc/{pid}/task/"))?
        .into_iter()
        .flatten()
        .filter(|d| d.path().is_dir())
        .map(|d| d.file_name().to_str().unwrap_or_default().parse())
        .collect::<Result<Vec<_>, _>>()?)
}

pub struct Mem<T>(pub T);

impl Mem<File> {
    pub fn open_process(pid: i32) -> Result<Self> {
        Ok(Mem(OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!("/proc/{pid}/mem"))?))
    }
}

impl<T> MemExt for Mem<T>
where
    T: FileExt,
{
    fn read_at(&self, offset: usize, size: usize) -> Result<Vec<u8>> {
        self.0.read_at(offset, size)
    }

    fn write_at(&self, offset: usize, payload: &[u8]) -> Result<usize> {
        self.0.write_at(offset, payload)
    }
}

pub trait FileExt {
    fn read_at(&self, offset: usize, size: usize) -> Result<Vec<u8>>;
    fn write_at(&self, offset: usize, payload: &[u8]) -> Result<usize>;
}

impl FileExt for File {
    fn read_at(&self, offset: usize, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; size];
        std::os::unix::prelude::FileExt::read_at(self, &mut buf, offset as u64)?;
        Ok(buf)
    }

    fn write_at(&self, offset: usize, payload: &[u8]) -> Result<usize> {
        Ok(std::os::unix::prelude::FileExt::write_at(self, payload, offset as u64)?)
    }
}
