use std::{
    fs, io,
    io::{BufWriter, Write},
    path::Path,
};

use bincode::{Decode, Encode};
use cli::{create_map::create_map, pointer_map::PointerMap, Map};

#[derive(Encode, Decode)]
pub struct PointerMapCache {
    pub map: PointerMap,
    pub region: Vec<Map>,
}

fn main() {
    
}

pub fn show_map_info<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    let name = path.as_ref().file_stem().map(|s| s.to_string_lossy()).unwrap();
    let size = path
        .as_ref()
        .extension()
        .and_then(|s| s.to_str().and_then(|s| s.parse::<usize>().ok()))
        .unwrap();

    let data = fs::read(path.as_ref())?;

    let mut buffer = BufWriter::new(std::io::stdout());

    for bin in data.chunks(size) {
        let (offset, path) = parse_line(bin);
        let path = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
        writeln!(buffer, "{name}+{:#x}->{path}", offset)?;
    }

    Ok(())
}

#[inline(always)]
pub fn parse_line(bin: &[u8]) -> (usize, impl Iterator<Item = i16> + '_) {
    let line = bin.rsplitn(2, |&n| n == 101).nth(1).unwrap();
    let (off, path) = line.split_at(8);
    let off = usize::from_le_bytes(off.try_into().unwrap());
    let path = path.chunks(2).rev().map(|x| i16::from_le_bytes(x.try_into().unwrap()));
    (off, path)
}
