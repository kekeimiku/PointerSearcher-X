use std::{num::ParseIntError, str::FromStr};

use crate::vmmap::{VirtualMemoryInfo, VirtualMemoryRead, VirtualQuery};

use super::error::Result;

pub fn bytes_to_usize(buf: &[u8]) -> Result<usize> {
    Ok(usize::from_le_bytes(buf.try_into().unwrap()))
}

pub fn parse_path(s: &str) -> Result<(&str, usize, Vec<i16>, i16)> {
    let error = "parse path error";
    let (name, last) = s.split_once('+').ok_or(error)?;
    let (off1, last) = last.split_once("->").ok_or(error)?;
    let off1 = usize::from_str_radix(off1.strip_prefix("0x").ok_or(error)?, 16)?;
    let (offv, last) = last.rsplit_once("->").ok_or(error)?;
    let offv = offv
        .split("->")
        .map(FromStr::from_str)
        .collect::<Result<Vec<i16>, ParseIntError>>()?;
    let last = last.parse()?;
    Ok((name, off1, offv, last))
}

pub const fn add(u: usize, i: i16) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as usize)
    } else {
        u.checked_add(i as usize)
    }
}

pub fn show_pointer_value<P>(proc: &P, path: &str) -> Result<()>
where
    P: VirtualMemoryRead + VirtualMemoryInfo,
{
    let (name, off1, offv, last) = parse_path(path)?;

    let mut address = proc
        .get_maps()
        // todo 保证rw
        .filter(|m| m.is_read() && m.is_write())
        .find(|m| m.path().ends_with(name))
        .map(|n| n.start() + off1)
        .ok_or("find modules error")?;

    let mut buf = vec![0; 8];

    for off in offv {
        proc.read_at(add(address, off).ok_or("error")?, &mut buf)?;
        address = bytes_to_usize(buf.as_mut_slice())?;
    }

    println!("{:#x}", add(address, last).ok_or("error")?);

    Ok(())
}
