use std::{fs::OpenOptions, io::BufWriter, mem, path::PathBuf};

use vmmap::{Pid, Process, ProcessInfo, VirtualQuery};

use crate::{
    consts::{BIN_CONFIG, MAX_BUF_SIZE},
    pointer_map::{create_pointer_map, PointerMap},
    spinner::Spinner,
    Map,
};

pub fn create_map(pid: Pid) {
    let proc = Process::open(pid).unwrap();

    let mut spinner = Spinner::default();
    spinner.start("开始创建指针缓存");

    let region = proc
        .get_maps()
        .filter(|m| m.is_read())
        .map(Map::from)
        .filter(|m| m.is_exe() || m.is_stack || m.is_heap || m.path.is_none())
        .collect::<Vec<_>>();

    let scan_region = region.iter().map(|m| (m.start, m.size)).collect::<Vec<_>>();
    let base_region = region.into_iter().filter(|m| m.path.is_some()).collect::<Vec<_>>();
    let base_region = merge_bases(base_region);

    let mut out_map = PointerMap::new();
    create_pointer_map(&proc, &scan_region, &mut out_map);
    let app_name = proc.app_path().file_name().unwrap();

    // 储存内存映射
    let path = PathBuf::new().with_file_name(app_name).with_extension("maps");
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();
    let mut writer = BufWriter::with_capacity(MAX_BUF_SIZE, file);
    bincode::encode_into_std_write(base_region, &mut writer, BIN_CONFIG).unwrap();

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
    spinner.stop("创建完成");
}

pub fn merge_bases(mut bases: Vec<Map>) -> Vec<Map> {
    let mut aom = Vec::new();
    let mut current = mem::take(&mut bases[0]);
    for map in bases.into_iter().skip(1) {
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
