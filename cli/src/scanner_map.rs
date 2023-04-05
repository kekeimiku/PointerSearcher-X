use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter},
    num::ParseIntError,
    path::Path,
};

use crate::{
    consts::{Address, PointerMap, BIN_CONFIG, MAX_BUF_SIZE},
    scanner::PointerSeacher,
    spinner::Spinner,
    Map,
};

pub fn calc_pointer_path<P: AsRef<Path>>(
    pointer_path: P,
    maps_path: P,
    target: Address,
    max_depth: usize,
    offset: (usize, usize),
) -> Result<(), io::Error> {
    let max_size = max_depth * 2 + 9;

    let file = File::open(maps_path)?;
    let mut reader = BufReader::with_capacity(MAX_BUF_SIZE, file);
    let maps: Vec<Map> = bincode::decode_from_std_read(&mut reader, BIN_CONFIG).unwrap();
    let maps = select_module(&maps).unwrap();

    let mut spinner = Spinner::default();
    spinner.start("Load pointer map...");

    let file = File::open(pointer_path)?;
    let mut reader = BufReader::with_capacity(MAX_BUF_SIZE, file);
    let pointer: PointerMap = bincode::decode_from_std_read(&mut reader, BIN_CONFIG).unwrap();

    let startpoints = pointer
        .keys()
        .copied()
        .filter(|a| maps.iter().any(|m| (m.start..m.end).contains(a)))
        .collect::<Vec<_>>();

    spinner.stop("load finished");

    let path = Path::new("./")
        .with_file_name(format!("{target:#x}"))
        .with_extension(max_size.to_string());
    let file = OpenOptions::new().write(true).append(true).create(true).open(path)?;
    let mut out = BufWriter::with_capacity(MAX_BUF_SIZE, file);

    let mut spinner = Spinner::default();
    spinner.start("Start calc pointer...");

    let mut ps = PointerSeacher::default();
    ps.load_map(pointer);
    ps.path_find_helpers(target, &mut out, offset, max_depth, &startpoints)
        .unwrap();
    spinner.stop("Calc finished");

    Ok(())
}

pub fn select_module(items: &[Map]) -> Result<Vec<Map>, ParseIntError> {
    let show: String = items
        .iter()
        .filter(|m| m.path.is_some())
        .enumerate()
        .map(|(k, v)| format!("[{k}: {v}] "))
        .collect();
    println!("{show}");
    println!("Select your module, separated by spaces");

    let mut selected_items = vec![];
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let input = input
        .split_whitespace()
        .map(|n| n.parse())
        .collect::<Result<Vec<usize>, _>>()?;

    for k in input {
        if k > items.len() {
            break;
        }
        selected_items.push(items[k].to_owned())
    }

    if selected_items.is_empty() {
        panic!("Select at least one")
    }

    Ok(selected_items)
}
