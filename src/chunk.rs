use std::{fs::File, io, os::unix::prelude::FileExt};

pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;
    fn next(&mut self) -> io::Result<Self::Item<'_>>;
}

pub struct Chunk<'a, R> {
    pub file: &'a R,
    pub idx: usize,
    pub buf: &'a mut [u8],
}

impl LendingIterator for Chunk<'_, File> {
    type Item<'a> = &'a [u8]
    where
        Self: 'a;

    fn next(&mut self) -> io::Result<Self::Item<'_>> {
        let size = self.file.read_at(self.buf, self.idx as _)?;
        if size == 0 {
            Err(io::ErrorKind::UnexpectedEof.into())
        } else {
            self.idx += size;
            Ok(&self.buf[..size])
        }
    }
}

impl<'a> Chunk<'a, File> {
    pub fn new(file: &'a File, buf: &'a mut [u8]) -> io::Result<Self> {
        Ok(Self { file, idx: 0, buf })
    }
}
