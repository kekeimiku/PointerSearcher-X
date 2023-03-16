use std::{fmt::Display, fs::OpenOptions, io};

use super::error::Result;
use crate::{
    consts::MAX_BUF_SIZE,
    pointer::Pointer,
    prompt::{select_modules, set_depth, set_offset, set_target},
    vmmap::{VirtualMemoryInfo, VirtualMemoryRead, VirtualQuery},
};

pub fn init_pointer_scanner<P>(proc: &P) -> Result<()>
where
    P: VirtualMemoryRead + Sync + Clone + VirtualMemoryInfo,
{
    let target = set_target()?;
    let depth = set_depth()?;
    let range = set_offset()?;

    let search_for = target
        .iter()
        .map(|a| {
            Ok((
                io::BufWriter::with_capacity(
                    MAX_BUF_SIZE,
                    OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(format!("{a:x}.bin"))?,
                ),
                *a,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut pointer_map = Pointer::default();

    let (scanner, watch) = get_scanner_region(proc.get_maps(), &target)?;

    pointer_map.create_map(proc, scanner.into_iter())?;
    pointer_map.find_path(&watch, *range, depth, search_for.into_iter(), None)?;

    Ok(())
}

pub fn get_scanner_region<I, V>(it: I, target: &[usize]) -> Result<(Vec<V>, Vec<V>)>
where
    I: Iterator<Item = V>,
    V: VirtualQuery + Clone + Display,
{
    let bases = it.filter(|m| m.is_read() && m.is_write()).collect::<Vec<V>>();
    let watch = bases.iter().filter(|m| !m.path().is_empty()).cloned().collect();
    let watch = select_modules(watch)?;
    let scanner = bases
        .into_iter()
        .filter(|m| {
            watch.iter().any(|x| (x.start()..x.end()).contains(&m.start()))
                || target.iter().any(|x| (m.start()..m.end()).contains(x))
        })
        .collect::<Vec<_>>();
    Ok((scanner, watch))
}
