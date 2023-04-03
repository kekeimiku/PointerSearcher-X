use std::{fs::OpenOptions, io::BufWriter, mem, path::PathBuf};

use vmmap::{Pid, Process, ProcessInfo, VirtualQuery};

use crate::{
    consts::{BIN_CONFIG, MAX_BUF_SIZE},
    pointer_map::{create_pointer_map, PointerMap},
    Map,
};

pub fn create_map(pid: Pid) {
    let proc = Process::open(pid).unwrap();
    let bases = proc
        .get_maps()
        .filter(|m| m.is_read())
        .map(Map::from)
        .collect::<Vec<_>>();

    // 根据一些规则默认选中一些区域
    let app_path = proc.app_path();

    let selections = bases
        .into_iter()
        .filter(|m| {
            m.path
                .as_ref()
                .map_or(false, |path| path.file_name().map_or(false, |f| f.eq("libhl.so")))
                || m.path.is_none()
        })
        .collect::<Vec<_>>();

    selections.iter().for_each(|m| println!("{m}"));

    // 要扫描的区域
    let scan_region = selections.iter().map(|m| (m.start, m.size)).collect::<Vec<_>>();
    // 合并过后的区域
    let out_region = merge_bases(&selections)
        .into_iter()
        .filter(|m| m.path.is_some())
        .collect::<Vec<_>>();

    let mut out_map = PointerMap::new();
    create_pointer_map(&proc, &scan_region, &mut out_map);

    let app_name = app_path.file_name().unwrap();

    // 储存内存映射
    let path = PathBuf::new().with_file_name(app_name).with_extension("maps");
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    let mut writer = BufWriter::with_capacity(MAX_BUF_SIZE, file);
    bincode::encode_into_std_write(out_region, &mut writer, BIN_CONFIG).unwrap();

    // 储存指针映射
    let path = PathBuf::new().with_file_name(app_name).with_extension("pointers");
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    let mut writer = BufWriter::with_capacity(MAX_BUF_SIZE, file);
    bincode::encode_into_std_write(out_map, &mut writer, BIN_CONFIG).unwrap();
}

fn merge_bases(bases: &[Map]) -> Vec<Map> {
    let mut tmp = bases.iter().filter(|m| m.path.is_some()).cloned().collect::<Vec<_>>();
    let mut aom = Vec::new();
    let mut current = mem::take(&mut tmp[0]);
    for map in tmp.into_iter().skip(1) {
        if map.path == current.path {
            current.end = map.end;
        } else {
            aom.push(current);
            current = map;
        }
    }
    aom.push(current);
    aom
}
