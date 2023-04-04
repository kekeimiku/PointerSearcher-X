use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter},
    num::ParseIntError,
    path::Path,
};

use crate::{
    consts::{Address, BIN_CONFIG, MAX_BUF_SIZE},
    pointer_map::{convert_rev_map, path_find_helpers, PointerMap},
    spinner::Spinner,
    Map,
};

pub fn scan<P: AsRef<Path>>(
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
    spinner.start("加载指针缓存");

    let file = File::open(pointer_path)?;
    let mut reader = BufReader::with_capacity(MAX_BUF_SIZE, file);
    let pointer: PointerMap = bincode::decode_from_std_read(&mut reader, BIN_CONFIG).unwrap();

    let startpoints = pointer
        .keys()
        .copied()
        .filter(|a| maps.iter().any(|m| (m.start..m.end).contains(a)))
        .collect::<Vec<_>>();

    spinner.stop("指针缓存加载完成");

    let path = Path::new("./")
        .with_file_name(target.to_string())
        .with_extension(max_size.to_string());
    let file = OpenOptions::new().write(true).append(true).create(true).open(path)?;
    let mut out = BufWriter::with_capacity(MAX_BUF_SIZE, file);

    spinner.start("开始查找路径");
    let rev_map = convert_rev_map(pointer);
    path_find_helpers(rev_map, target, &mut out, offset, max_depth, &startpoints).unwrap();
    spinner.stop("路径查找完成");

    Ok(())
}

pub fn select_module(items: &[Map]) -> Result<Vec<Map>, ParseIntError> {
    let show: String = items
        .iter()
        .filter(|m| m.path.is_some())
        .enumerate()
        .map(|(k, v)| format!("[{k}]: {v} "))
        .collect();
    println!("{show}");
    println!("选择你关心的区域");

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

    Ok(selected_items)
}
