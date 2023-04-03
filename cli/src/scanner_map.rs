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

pub fn scan<P: AsRef<Path>>(pointer_path: P, maps_path: P) -> Result<(), io::Error> {
    let target = 0x76747de8;
    let max_depth = 7;
    let offset = (0, 800);

    let max_size = max_depth * 2 + 9;

    let file = File::open(maps_path)?;
    let mut reader = BufReader::with_capacity(MAX_BUF_SIZE, file);
    let maps: Vec<Map> = bincode::decode_from_std_read(&mut reader, BIN_CONFIG).unwrap();
    maps.iter().for_each(|m| println!("{m}"));

    let file = File::open(pointer_path)?;
    let mut reader = BufReader::with_capacity(MAX_BUF_SIZE, file);
    let pointer: PointerMap = bincode::decode_from_std_read(&mut reader, BIN_CONFIG).unwrap();

    let startpoints = pointer
        .keys()
        .copied()
        .filter(|a| maps.iter().any(|m| (m.start..m.end).contains(a)))
        .collect::<Vec<_>>();

    println!("len {}", startpoints.len());

    let path = Path::new("./")
        .with_file_name(target.to_string())
        .with_extension(max_size.to_string());
    let file = OpenOptions::new().write(true).append(true).create(true).open(path)?;
    let mut out = BufWriter::with_capacity(MAX_BUF_SIZE, file);

    let rev_map = convert_rev_map(pointer);
    path_find_helpers(rev_map, target, &mut out, offset, max_depth, &startpoints)
}
