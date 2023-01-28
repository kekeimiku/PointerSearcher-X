use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::{
    cache::{Chunks, Offset, BUFFERSIZE},
    error::Result,
    proc::{MapsExt, MemExt},
};

// 扫描模式
#[derive(Debug, Clone, Copy)]
pub enum ScanMode {
    Fast,
    Normal,
}

#[derive(Debug, Clone, Copy)]
pub enum ScanType {
    Exact,
    Less,
    More,
    Between,
    Unknown,
}

// 初次搜索，从data中找到target的位置
pub fn find_vec(data: Vec<u8>, target: &[u8], mode: ScanMode) -> Vec<Offset> {
    match mode {
        ScanMode::Fast => data
            .windows(target.len())
            .enumerate()
            .step_by(target.len())
            .filter_map(|(k, v)| if v == target { Some(k as Offset) } else { None })
            .collect(),
        ScanMode::Normal => memchr::memmem::find_iter(&data, target).map(|n| n as _).collect(),
    }
}

// 之后的搜索，遍历addr从data中检查值是否为target
pub fn refind_vec(addr: Vec<Offset>, data: Vec<u8>, target: &[u8]) -> Vec<Offset> {
    addr.into_iter()
        .filter(|addr| {
            let addr = *addr as usize;
            &data[addr..addr + target.len()] == target
        })
        .collect()
}

// 从副本中搜索
pub fn scan<'a, M: MapsExt>(
    map: &'a M,
    file: File,
    target: &'a [u8],
    mode: ScanMode,
) -> impl Iterator<Item = Result<Vec<Offset>>> + 'a {
    Chunks::new(file, 0, map.end() - map.start(), BUFFERSIZE)
        .map(move |data| Ok(find_vec(data?, target, mode)))
}

// 重新搜索
pub fn rescan<'a, A: MapsExt, B: MemExt>(
    map: &A,
    mem: &'a B,
    target: &'a [u8],
    addr: impl Iterator<Item = Result<Vec<Offset>>> + 'a,
) -> impl Iterator<Item = Result<Vec<Offset>>> + 'a {
    Chunks::new(mem, 0, map.end() - map.start(), BUFFERSIZE)
        .zip(addr)
        .map(move |(data, addr)| Ok(refind_vec(addr?, data?, target)))
}

// 保存内存副本
pub fn dump_mem<A: MapsExt, B: MemExt, P: AsRef<Path>>(maps: &A, mem: &B, path: P) -> Result<()> {
    let mut file = OpenOptions::new().append(true).create(true).open(path)?;
    for data in Chunks::new(mem, maps.start(), maps.end(), BUFFERSIZE) {
        file.write_all(&data?)?;
    }

    Ok(())
}
