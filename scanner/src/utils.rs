use std::{array::TryFromSliceError, io, num::ParseIntError};

use crate::map::Map;

pub fn bytes_to_usize(buf: &[u8]) -> Result<usize, String> {
    Ok(usize::from_le_bytes(buf.try_into().map_err(|e: TryFromSliceError| e.to_string())?))
}

pub const fn wrap_add(u: usize, i: i16) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as usize)
    } else {
        u.checked_add(i as usize)
    }
}

pub fn select_module(items: Vec<Map>) -> Result<Vec<Map>, ParseIntError> {
    let items = crate::utils::merge_bases(items);

    let show: String = items
        .iter()
        .filter_map(|m| m.path.file_name())
        .enumerate()
        .map(|(k, v)| format!("[{k}: {}] ", v.to_string_lossy()))
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

pub fn merge_bases(mut bases: Vec<Map>) -> Vec<Map> {
    let mut aom = Vec::new();
    let mut current = core::mem::take(&mut bases[0]);
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

// impl SubCommandSPV {
//     pub fn init(self) -> Result<()> {
//         let SubCommandSPV { pid, path } = self;
//         let proc = Process::open(pid)?;
//         let (name, off, offv, last) = parse_path(&path).ok_or("err")?;
//         let mut address = proc
//             .get_maps()
//             .filter(|m| m.is_read() && m.path().is_some())
//             .find(|m| m.path().map_or(false, |f| f.file_name().map_or(false,
// |n| n.eq(name))))             .map(|m| m.start() + off)
//             .ok_or("find modules error")
//             .unwrap();

//         let mut buf = vec![0; 8];

//         for off in offv {
//             proc.read_at(wrap_add(address, off).ok_or("err")?, &mut buf)?;
//             address = bytes_to_usize(buf.as_mut_slice())?;
//         }

//         println!("{:#x}", wrap_add(address, last).ok_or("err")?);

//         Ok(())
//     }
// }

// #[inline(always)]
// pub fn parse_path(path: &str) -> Option<(&str, usize, Vec<i16>, i16)> {
//     let (name, last) = path.split_once('+')?;
//     let (off1, last) = last.split_once("->")?;
//     let off1 = usize::from_str_radix(off1.strip_prefix("0x")?, 16).ok()?;
//     let (offv, last) = last.rsplit_once("->")?;
//     let offv = offv
//         .split("->")
//         .map(FromStr::from_str)
//         .collect::<Result<Vec<i16>, ParseIntError>>()
//         .ok()?;
//     let last = last.parse().ok()?;
//     Some((name, off1, offv, last))
// }
