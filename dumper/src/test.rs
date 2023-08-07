#[cfg(target_os = "linux")]
use vmmap::linux::VirtualQueryExt;
#[cfg(target_os = "macos")]
use vmmap::macos::VirtualQueryExt;
#[cfg(target_os = "windows")]
use vmmap::windows::VirtualQueryExt;
use vmmap::{Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::{Error, SubCommandTest};

#[cfg(target_os = "linux")]
fn find_base_address<P: ProcessInfo>(proc: &P, name: &str) -> Result<usize, &'static str> {
    use std::path::Path;

    proc.get_maps()
        .filter(|m| m.is_read() && !m.name().is_empty())
        .find(|m| Path::new(m.name()).file_name().map_or(false, |n| n.eq(name)))
        .map(|m| m.start())
        .ok_or("find modules error")
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn find_base_address<P: ProcessInfo>(proc: &P, name: &str) -> Result<usize, &'static str> {
    proc.get_maps()
        .filter(|m| m.is_read() && m.path().is_some())
        .find(|m| m.path().and_then(|f| f.file_name()).is_some_and(|n| n.eq(name)))
        .map(|m| m.start())
        .ok_or("find modules error")
}

impl SubCommandTest {
    pub fn init(self) -> Result<(), Error> {
        let SubCommandTest { pid, path, num } = self;
        let proc = Process::open(pid)?;
        let (name, offv, last) = parse_path(&path).ok_or("parse path error")?;
        let mut address = find_base_address(&proc, name)? as usize;

        #[cfg(target_pointer_width = "32")]
        let mut buf = [0; 4];
        #[cfg(target_pointer_width = "64")]
        let mut buf = [0; 8];

        for off in offv {
            proc.read_at(&mut buf, wrap_add(address, off)?)?;
            address = usize::from_le_bytes(buf);
        }

        let address = wrap_add(address, last)?;
        println!("{address:#x}");

        if let Some(num) = num {
            let mut buf = vec![0; num];
            proc.read_at(&mut buf, address)?;
            println!("{}", buf.iter().map(|x| format!("{x:02x}")).collect::<String>());
        }

        Ok(())
    }
}

#[inline(always)]
fn parse_path(path: &str) -> Option<(&str, Vec<isize>, isize)> {
    let (name, last) = path.split_once('+')?;
    let (offv, last) = last.rsplit_once('@')?;
    let offv = offv
        .split('@')
        .map(|x| x.parse())
        .collect::<Result<Vec<isize>, _>>()
        .ok()?;
    let last = last.parse().ok()?;
    Some((name, offv, last))
}

#[inline(always)]
fn wrap_add(u: usize, i: isize) -> Result<usize, &'static str> {
    u.checked_add_signed(i).ok_or("pointer overflow")
}
