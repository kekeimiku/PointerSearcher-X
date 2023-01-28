use std::{fs::File, io::ErrorKind};

// 这个trait 需要实现类似pread pwrite的功能
pub trait ReaderExt {
    fn read_at(&self, offset: usize, size: usize) -> std::io::Result<Vec<u8>>;
    fn write_at(&self, offset: usize, payload: &[u8]) -> std::io::Result<usize>;
}

#[cfg(target_family = "unix")]
use std::os::unix::prelude::FileExt;

#[cfg(target_family = "unix")]
impl ReaderExt for File {
    fn read_at(&self, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0; size];
        FileExt::read_exact_at(self, &mut buf, offset as u64)?;
        Ok(buf)
    }

    fn write_at(&self, offset: usize, data: &[u8]) -> std::io::Result<usize> {
        std::os::unix::prelude::FileExt::write_at(self, data, offset as u64)
    }
}

#[cfg(target_family = "windows")]
impl ReaderExt for File {
    fn read_at(&self, offset: usize, size: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0; size];
        std::os::windows::prelude::FileExt::seek_read(self, &mut buf, offset as u64)?;
        Ok(buf)
    }

    fn write_at(&self, offset: usize, data: &[u8]) -> std::io::Result<usize> {
        Ok(std::os::windows::prelude::FileExt::seek_write(self, data, offset as u64)?)
    }
}
