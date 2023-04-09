use std::{array::TryFromSliceError, io, num::ParseIntError, path::PathBuf};

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

pub fn select_module(items: Vec<(usize, usize, PathBuf)>) -> Result<Vec<(usize, usize, PathBuf)>, ParseIntError> {
    let items = crate::utils::merge_bases(items);

    let show: String = items
        .iter()
        .filter_map(|(_, _, path)| path.file_name())
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

pub fn merge_bases(mut bases: Vec<(usize, usize, PathBuf)>) -> Vec<(usize, usize, PathBuf)> {
    let mut aom = Vec::new();
    let mut current = core::mem::take(&mut bases[0]);
    for map in bases.into_iter().skip(1) {
        if map.2 == current.2 {
            current.1 = map.1;
        } else {
            aom.push(current);
            current = map;
        }
    }
    aom.push(current);
    aom
}
