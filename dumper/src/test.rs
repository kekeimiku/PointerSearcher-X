use std::{num::ParseIntError, str::FromStr};

use vmmap::vmmap64::{Process, ProcessInfo, VirtualMemoryRead, VirtualQuery, VirtualQueryExt};

use super::cmd::SubCommandTest;

#[cfg(target_os = "linux")]
pub fn find_base_address<P: ProcessInfo>(proc: &P, name: &str) -> Result<u64, &'static str> {
    proc.get_maps()
        .filter(|m| m.is_read() && !m.name().is_empty())
        .find(|m| m.name().eq(name))
        .map(|m| m.start())
        .ok_or("find modules error")
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn find_base_address<P: ProcessInfo>(proc: &P, name: &str) -> Result<u64, &'static str> {
    proc.get_maps()
        .filter(|m| m.is_read() && m.path().is_some())
        .find(|m| m.path().and_then(|f| f.file_name()).is_some_and(|n| n.eq(name)))
        .map(|m| m.start())
        .ok_or("find modules error")
}

impl SubCommandTest {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandTest { pid, path, num } = self;
        let proc = Process::open(pid)?;
        let (name, off, offv, last) = parse_path(&path).ok_or("parse path error")?;
        let mut address = find_base_address(&proc, name)? as usize + off;

        let mut buf = [0; 8];

        for off in offv {
            proc.read_at(wrap_add(address, off)? as u64, &mut buf)?;
            address = usize::from_le_bytes(buf);
        }

        let address = wrap_add(address, last)?;
        println!("{address:#x}");

        if let Some(num) = num {
            let mut buf = vec![0; num];
            proc.read_at(address as u64, &mut buf)?;
            println!("{}", buf.iter().map(|x| format!("{x:02x}")).collect::<String>());
        }

        Ok(())
    }
}

#[inline(always)]
fn parse_path(path: &str) -> Option<(&str, usize, Vec<isize>, isize)> {
    let (name, last) = path.split_once('+')?;
    let (off1, last) = last.split_once('@')?;
    let off1 = off1.parse().ok()?;
    let (offv, last) = last.rsplit_once('@')?;
    let offv = offv
        .split('@')
        .map(FromStr::from_str)
        .collect::<Result<Vec<isize>, ParseIntError>>()
        .ok()?;
    let last = last.parse().ok()?;
    Some((name, off1, offv, last))
}

#[inline(always)]
pub fn wrap_add(u: usize, i: isize) -> Result<usize, &'static str> {
    add(u, i).ok_or("pointer overflow")
}

#[inline(always)]
const fn add(u: usize, i: isize) -> Option<usize> {
    if i.is_negative() {
        u.checked_sub(i.wrapping_abs() as usize)
    } else {
        u.checked_add(i as usize)
    }
}
