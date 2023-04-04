use std::{
    fs::{self, File},
    io,
    io::{BufWriter, Write},
    path::Path,
};

use bincode::{Decode, Encode};
use cli::{
    cmd::{CommandEnum, Commands},
    consts::BIN_CONFIG,
    create_map::create_map,
    pointer_map::PointerMap,
    pointer_path::show_pointer_value,
    scanner_map::calc_pointer_path,
    Map,
};

#[derive(Encode, Decode)]
pub struct PointerMapCache {
    pub map: PointerMap,
    pub region: Vec<Map>,
}

fn main() {
    let args: Commands = argh::from_env();
    match args.nested {
        CommandEnum::CreatePointerMap(args) => create_map(args.pid),
        CommandEnum::CalcPointerPath(args) => {
            calc_pointer_path(args.pf, args.mf, *args.target, args.depth, *args.offset).unwrap();
        }
        CommandEnum::ShowPointerPath(args) => show_map_info(args.rf, args.mf).unwrap(),
        CommandEnum::ShowPointerPathValue(args) => show_pointer_value(args.pid, &args.path),
    }
}

pub fn show_map_info<P: AsRef<Path>>(path: P, maps_path: P) -> Result<(), io::Error> {
    let size = path
        .as_ref()
        .extension()
        .and_then(|s| s.to_str().and_then(|s| s.parse::<usize>().ok()))
        .unwrap();

    let data = fs::read(path.as_ref())?;

    let mut file = File::open(maps_path)?;
    let maps: Vec<Map> = bincode::decode_from_std_read(&mut file, BIN_CONFIG).unwrap();

    let mut buffer = BufWriter::new(std::io::stdout());

    for bin in data.chunks(size) {
        let (offset, path) = parse_line(bin);
        let path = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
        for map in maps.iter().filter(|m| m.path.is_some()) {
            if (map.start..map.end).contains(&offset) {
                let name = map.path.as_ref().and_then(|f| f.file_name()).unwrap().to_string_lossy();
                writeln!(buffer, "{name}+{:#x}->{path}", offset - map.start)?;
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
