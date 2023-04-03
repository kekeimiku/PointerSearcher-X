use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter},
    path::Path,
};

use crate::{
    consts::{BIN_CONFIG, MAX_BUF_SIZE},
    pointer_map::{convert_rev_map, path_find_helpers, PointerMap},
    Map,
};

pub fn scan(pointer_path: &Path, maps_path: &Path) -> Result<(), io::Error> {
    let target = 0x60000271c124;
    let max_depth = 11;
    let offset = (128, 128);

    
    let max_size = max_depth * 2 + 9;

    let file = File::open(maps_path)?;
    let mut reader = BufReader::with_capacity(MAX_BUF_SIZE, file);
    let maps: Vec<Map> = bincode::decode_from_std_read(&mut reader, BIN_CONFIG).unwrap();

    let file = File::open(pointer_path)?;
    let mut reader = BufReader::with_capacity(MAX_BUF_SIZE, file);
    let pointer: PointerMap = bincode::decode_from_std_read(&mut reader, BIN_CONFIG).unwrap();

    let startpoints = pointer
        .keys()
        .copied()
        .filter(|a| maps.iter().any(|m| (m.start..m.end).contains(a)))
        .collect::<Vec<_>>();

    let path = Path::new("./")
        .with_file_name(target.to_string())
        .with_extension(max_size.to_string());
    let file = OpenOptions::new().write(true).append(true).create(true).open(path)?;
    let mut out = BufWriter::with_capacity(MAX_BUF_SIZE, file);

    let rev_map = convert_rev_map(pointer);
    path_find_helpers(rev_map, target, &mut out, offset, max_depth, &startpoints)
}
