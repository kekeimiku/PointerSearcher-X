use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io,
    io::{BufWriter, Write},
    ops::Bound::Included,
    path::PathBuf,
};

use argh::{FromArgValue, FromArgs};
use consts::{Address, MAX_BUF_SIZE};

use crate::{
    b::{convert_bin_to_txt, load_pointer_map},
    e::PointerSeacher,
    map::Map,
    utils::select_module,
};

pub struct Target(Address);

impl FromArgValue for Target {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let value = value.trim_start_matches("0x");
        let address = Address::from_str_radix(value, 16).map_err(|e| e.to_string())?;
        Ok(Self(address))
    }
}

pub struct Offset((usize, usize));

impl FromArgValue for Offset {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let (lr, ur) = value.split_once(':').ok_or("err")?;
        let lr = lr.trim_start_matches('-').parse::<usize>().map_err(|e| e.to_string())?;
        let ur = ur.trim_start_matches('+').parse::<usize>().map_err(|e| e.to_string())?;
        Ok(Self((lr, ur)))
    }
}

#[derive(FromArgs)]
#[argh(description = "Commands.")]
pub struct Commands {
    #[argh(option, short = 'f', description = "file path")]
    pub file: PathBuf,
    #[argh(option, short = 't', description = "target address")]
    pub target: Target,
    #[argh(option, default = "7", short = 'd', description = "depth")]
    pub depth: usize,
    #[argh(option, default = "Offset((0, 600))", short = 'o', description = "offset")]
    pub offset: Offset,
    #[argh(option, default = "PathBuf::new()", description = "out dir path")]
    pub dir: PathBuf,
}

impl Commands {
    pub fn init(self) -> io::Result<()> {
        let Commands { file, target, dir, depth, offset } = self;
        let (pmap, mmap) = load_pointer_map(file)?;
        let select = select_module(mmap).unwrap();
        let points = select
            .iter()
            .flat_map(|Map { start, end, path: _ }| pmap.range((Included(start), Included(end))).map(|(&k, _)| k))
            .collect::<Vec<_>>();
        let mut map: BTreeMap<Address, Vec<Address>> = BTreeMap::new();
        for (k, v) in pmap {
            map.entry(v).or_default().push(k);
        }

        let s = select
            .iter()
            .map(|Map { start, end, path }| format!("{start}-{end}-{}", path.to_string_lossy()))
            .collect::<String>();

        let path = dir.join("out.bin");

        let mut out = BufWriter::with_capacity(
            MAX_BUF_SIZE,
            OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(&path)
                .unwrap(),
        );

        out.write_all(&s.len().to_le_bytes())?;
        out.write_all(s.as_bytes())?;

        PathFindEngine {
            target: target.0,
            depth,
            offset: offset.0,
            out: &mut out,
            startpoints: points,
            engine: PointerSeacher(map),
        }
        .find_pointer_path()?;

        convert_bin_to_txt(path)
    }
}

pub struct PathFindEngine<'a, W> {
    target: Address,
    depth: usize,
    offset: (usize, usize),
    out: &'a mut W,
    startpoints: Vec<Address>,
    engine: PointerSeacher,
}

impl<W> PathFindEngine<'_, W>
where
    W: io::Write,
{
    pub fn find_pointer_path(self) -> io::Result<()> {
        let PathFindEngine { target, depth, offset, out, engine, startpoints } = self;
        let size = depth * 2 + 9;
        out.write_all(&size.to_le_bytes())?;
        engine.path_find_helpers(target, out, offset, depth, size, &startpoints)?;
        Ok(())
    }
}
