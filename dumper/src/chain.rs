use std::{fmt::Write, mem, path::Path};

use vmmap::{Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::{ChainCommand, Error};

fn find_base_address<P: ProcessInfo>(proc: &P, name: &str) -> Result<usize, &'static str> {
    proc.get_maps()
        .filter(|m| m.is_read())
        .find(|m| {
            m.name()
                .is_some_and(|s| Path::new(s).file_name().is_some_and(|n| n.eq(name)))
        })
        .map(|m| m.start())
        .ok_or("find modules error")
}

impl ChainCommand {
    pub fn init(self) -> Result<(), Error> {
        let ChainCommand { pid, chain: path, num } = self;
        let proc = Process::open(pid)?;
        let (name, offv, last) = parse_path(&path).ok_or("parse pointer chain error")?;
        let mut address = find_base_address(&proc, name)?;

        let mut buf = [0; mem::size_of::<usize>()];

        for off in offv {
            proc.read_at(&mut buf, address.checked_add_signed(off).ok_or("pointer overflow")?)?;
            address = usize::from_le_bytes(buf);
        }

        let address = address.checked_add_signed(last).ok_or("pointer overflow")?;
        println!("{address:#x}");

        if let Some(num) = num {
            let mut buf = vec![0; num];
            proc.read_at(&mut buf, address)?;
            println!("{}", hex_encode(&buf));
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
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(256), |mut output, b| {
        let _ = write!(output, "{b:02X}");
        output
    })
}
