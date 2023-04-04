use std::{num::ParseIntError, str::FromStr};

use vmmap::{Pid, Process, VirtualMemoryRead, VirtualQuery};

use crate::{bytes_to_usize, wrap_add};

pub fn parse_path(input: &str) -> (&str, usize, Vec<i16>, i16) {
    let (name, last) = input.split_once('+').unwrap();
    let (off1, last) = last.split_once("->").unwrap();
    let off1 = usize::from_str_radix(off1.strip_prefix("0x").unwrap(), 16).unwrap();
    let (offv, last) = last.rsplit_once("->").unwrap();
    let offv = offv
        .split("->")
        .map(FromStr::from_str)
        .collect::<Result<Vec<i16>, ParseIntError>>()
        .unwrap();
    let last = last.parse().unwrap();
    (name, off1, offv, last)
}

pub fn show_pointer_value(pid: Pid, input: &str) {
    let proc = Process::open(pid).unwrap();
    let (name, off, offv, last) = parse_path(input);

    let mut address = proc
        .get_maps()
        .filter(|m| m.is_read() && m.path().is_some())
        .find(|m| m.path().map_or(false, |f| f.file_name().map_or(false, |n| n.eq(name))))
        .map(|m| m.start() + off)
        .ok_or("find modules error")
        .unwrap();

    let mut buf = vec![0; 8];

    for off in offv {
        proc.read_at(wrap_add(address, off).unwrap(), &mut buf).unwrap();
        address = bytes_to_usize(buf.as_mut_slice()).unwrap();
    }

    println!("{:#x}", wrap_add(address, last).unwrap());
}
