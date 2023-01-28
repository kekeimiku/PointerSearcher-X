use crate::{
    error::Result,
    proc::{FileExt, MemExt},
    utils,
};
use core::ops::Range;
use std::{
    fs::{create_dir, remove_dir_all, File},
    io::{Read, Write},
    path::{Path},
};

pub type Offset = u32;

// 最大缓冲区大小，不能超过 Offset::MAX
pub const BUFFERSIZE: usize = 5120000;

pub const CACHE_DIR_NAME: &str = "LINCE_CACHE";
pub const MEMORY_TMP: &str = "Memory.TMP";
pub const MEMORY_UNDO: &str = "Memory.UNDO";
pub const ADDRESS_TMP: &str = "Address.TMP";
pub const ADDRESS_UNDO: &str = "Address.UNDO";

// pub static TMP_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
//     let tmp_dir = PathBuf::from("./");
//     if let Err(err) = init_cache_dir(&tmp_dir) {
//         println!("{err}")
//     };
//     tmp_dir
// });

pub struct Chunks<T> {
    handle: T,
    start: usize,
    size: usize,
    num: usize,
    last: usize,
}

impl<T> Chunks<T> {
    pub fn new(handle: T, start: usize, end: usize, mut size: usize) -> Self {
        let mut last = 0;
        let mut num = 1;
        assert!(end > start, "seek error");
        if size < end - start {
            num = (end - start) / size;
            last = (end - start) % size;
        } else {
            size = end - start;
        }

        Self { handle, start, size, num, last }
    }
}

impl<T: MemExt> Iterator for Chunks<&T> {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num != 0 {
            match self.handle.read_at(self.start, self.size) {
                Ok(chunk) => {
                    self.start += self.size;
                    self.num -= 1;
                    return Some(Ok(chunk));
                }
                Err(err) => return Some(Err(err)),
            };
        }

        if self.last != 0 {
            match self.handle.read_at(self.start, self.last) {
                Ok(chunk) => {
                    self.last = 0;
                    return Some(Ok(chunk));
                }
                Err(err) => return Some(Err(err)),
            };
        }

        None
    }
}

impl Iterator for Chunks<File> {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num != 0 {
            match self.handle.read_at(self.start, self.size) {
                Ok(chunk) => {
                    self.start += self.size;
                    self.num -= 1;
                    return Some(Ok(chunk));
                }
                Err(err) => return Some(Err(err)),
            };
        }

        if self.last != 0 {
            match self.handle.read_at(self.start, self.last) {
                Ok(chunk) => {
                    self.last = 0;
                    return Some(Ok(chunk));
                }
                Err(err) => return Some(Err(err)),
            };
        }

        None
    }
}

#[inline(always)]
pub fn write_cache<W: Write, T>(
    iter: impl Iterator<Item = Result<Vec<T>>>,
    mut writer: W,
) -> Result<(Vec<Range<usize>>, usize)> {
    let mut start = 0;
    let mut end;
    let mut result = Vec::new();
    let mut size = 0;

    for data in iter {
        let data = data?;
        size += data.len();
        let data = utils::vec_as_bytes(&data);
        let size = writer.write(data)?;
        end = size + start;
        result.push(start..end);
        start = end;
    }

    Ok((result, size))
}

#[inline(always)]
pub fn read_cache<'a, T, R: Read + 'a>(
    mut reader: R,
    index: &'a [Range<usize>],
) -> impl Iterator<Item = Result<Vec<T>>> + 'a {
    index.iter().map(move |i| {
        let mut buf = vec![0_u8; i.end - i.start];
        reader.read_exact(&mut buf)?;
        Ok(utils::vec_from_bytes(buf))
    })
}

// 如果存在cache目录则清空，否则创建
#[inline(always)]
pub fn init_cache_dir<P: AsRef<Path>>(tmp_dir: P) -> std::io::Result<()> {
    let dir_path_buf = tmp_dir.as_ref().join(CACHE_DIR_NAME);
    match dir_path_buf.is_dir() {
        true => remove_dir_all(&dir_path_buf).and_then(|_| create_dir(dir_path_buf)),
        false => create_dir(dir_path_buf),
    }
}
