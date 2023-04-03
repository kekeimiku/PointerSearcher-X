use std::{
    fs::{self, File},
    io::{self, BufReader},
    io::{BufWriter, Write},
    path::Path,
};

use bincode::{Decode, Encode};

use cli::{
    consts::{BIN_CONFIG, MAX_BUF_SIZE},
    pointer_map::PointerMap,
    Map,
};

#[derive(Encode, Decode)]
pub struct PointerMapCache {
    pub map: PointerMap,
    pub region: Vec<Map>,
}

fn main() {}

pub fn show_map_info<P: AsRef<Path>>(ptr_path: P, maps_path: P) -> Result<(), io::Error> {
    let size = ptr_path
        .as_ref()
        .extension()
        .and_then(|s| s.to_str().and_then(|s| s.parse::<usize>().ok()))
        .unwrap();

    let file = File::open(maps_path)?;
    let mut reader = BufReader::with_capacity(MAX_BUF_SIZE, file);
    let maps: Vec<Map> = bincode::decode_from_std_read(&mut reader, BIN_CONFIG).unwrap();

    let data = fs::read(ptr_path.as_ref())?;

    let mut stdout = BufWriter::new(std::io::stdout());

    for bin in data.chunks(size) {
        let (offset, path) = parse_line(bin);
        let path = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
        for map in &maps {
            let name = map
                .path
                .as_ref()
                .and_then(|n| n.file_name())
                .map(|n| n.to_string_lossy())
                .unwrap_or_else(|| "err".into());
            if (map.start..map.end).contains(&offset) {
                writeln!(stdout, "{name}+{:#x}->{path}", offset - map.start)?;
            }
        }
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
