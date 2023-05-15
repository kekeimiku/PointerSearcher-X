use std::{num::ParseIntError, str::FromStr};

use utils::consts::{Address, POINTER_SIZE};
use vmmap::{Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::cmd::SubCommandTest;

impl SubCommandTest {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandTest { pid, path, num } = self;
        let proc = Process::open(pid)?;
        let (name, off, offv, last) = wrap_parse_path(&path)?;
        let mut address = proc
            .get_maps()
            .filter(|m| m.is_read() && m.path().is_some())
            .find(|m| m.path().and_then(|f| f.file_name()).map_or(false, |n| n.eq(name)))
            .map(|m| m.start() + off)
            .ok_or("find modules error")?;

        let mut buf = [0; POINTER_SIZE];

        for off in offv {
            proc.read_at(wrap_add(address, off)? as _, &mut buf)?;
            address = Address::from_le_bytes(buf);
        }

        let address = wrap_add(address, last)?;
        println!("{address:#x}");

        if let Some(num) = num {
            let mut buf = vec![0; num];
            proc.read_at(address as _, &mut buf)?;
            println!("{}", buf.iter().map(|x| format!("{x:02x}")).collect::<String>());
        }

        Ok(())
    }
}

#[inline(always)]
pub fn wrap_parse_path(path: &str) -> Result<(&str, usize, Vec<i16>, i16), &'static str> {
    parse_path(path).ok_or("parse path err")
}

#[inline(always)]
fn parse_path(path: &str) -> Option<(&str, usize, Vec<i16>, i16)> {
    let (name, last) = path.split_once('+')?;
    let (off1, last) = last.split_once("->")?;
    let off1 = usize::from_str_radix(off1.strip_prefix("0x")?, 16).ok()?;
    let (offv, last) = last.rsplit_once("->")?;
    let offv = offv
        .split("->")
        .map(FromStr::from_str)
        .collect::<Result<Vec<i16>, ParseIntError>>()
        .ok()?;
    let last = last.parse().ok()?;
    Some((name, off1, offv, last))
}

#[inline(always)]
pub fn wrap_add(u: usize, i: i16) -> Result<usize, &'static str> {
    add(u, i).ok_or("pointer overflow")
}

#[inline(always)]
const fn add(u: usize, i: i16) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as usize)
    } else {
        u.checked_add(i as usize)
    }
}
