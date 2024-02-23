use std::{fmt::Write, mem, path::Path, process};

use vmmap::{Process, ProcessInfo, VirtualMemoryRead, VirtualMemoryWrite, VirtualQuery};

use super::{Error, TestChainCommand};

impl TestChainCommand {
    pub fn init(self) -> Result<(), Error> {
        let TestChainCommand { pid, chain, write, read } = self;
        let proc = Process::open(pid)?;
        let address = get_pointer_chain_address(&proc, chain).ok_or("Invalid pointer chain")?;
        println!("target = {address:x}");

        if let Some(size) = read {
            let mut buf = vec![0; size];
            proc.read_exact_at(&mut buf, address)?;
            println!("{}", hex_encode(&buf));
        }

        if let Some(bytes) = write {
            proc.write_at(&bytes.0, address)?;
        }

        Ok(())
    }
}

#[inline]
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(256), |mut output, b| {
        let _ = write!(output, "{b:02X}");
        output
    })
}

#[inline]
pub fn get_pointer_chain_address<P, S>(proc: &P, chain: S) -> Option<usize>
where
    P: VirtualMemoryRead + ProcessInfo,
    S: AsRef<str>,
{
    let mut parts = chain.as_ref().split(['[', ']', '+', '@']).filter(|s| !s.is_empty());
    let name = parts.next()?;
    let index = parts.next()?.parse().ok()?;
    let offset = parts.next_back()?.parse().ok()?;
    let elements = parts.map(|s| s.parse());

    let mut address = find_base_address(proc, name, index).unwrap_or_else(|| {
        println!("module not found: {name}[{index}]");
        process::exit(0);
    });

    println!("{name}[{index}] + {offset} = {address:x}");
    let mut buf = [0; mem::size_of::<usize>()];
    for element in elements {
        let element = element.ok()?;
        proc.read_exact_at(&mut buf, address.checked_add_signed(element)?)
            .ok()?;
        address = usize::from_le_bytes(buf);
        println!("+ {element} = {address:x}");
    }

    let target = address.checked_add_signed(offset);

    if let Some(addr) = target {
        println!("+ {offset} = {addr}");
    }

    target
}

struct Module<'a> {
    start: usize,
    end: usize,
    name: &'a str,
}

#[inline]
fn find_base_address<P: ProcessInfo>(proc: &P, name: &str, index: usize) -> Option<usize> {
    let vqs = proc.get_maps().flatten().collect::<Vec<_>>();
    vqs.iter()
        .filter(|x| x.is_write() && x.is_read())
        .flat_map(|x| Some(Module { start: x.start(), end: x.end(), name: x.name()? }))
        .fold(Vec::<Module>::with_capacity(vqs.len()), |mut acc, cur| {
            match acc.last_mut() {
                Some(last) if last.name == cur.name => last.end = cur.end,
                _ => acc.push(cur),
            }
            acc
        })
        .into_iter()
        .map(|Module { start, end, name }| {
            let name = Path::new(name).file_name().and_then(|s| s.to_str()).unwrap_or(name);
            Module { start, end, name }
        })
        .filter(|x| x.name.eq(name))
        .nth(index)
        .map(|x| x.start)
}
