use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    num::ParseIntError,
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};

use argh::{FromArgValue, FromArgs};
use ptrsx::{
    consts::{Address, MAX_BUF_SIZE},
    pointer_map::{ptrsx_create_pointer_map, ptrsx_decode_maps},
    pointer_path::{ptrsx_init_engine, Filter, PathFindParams},
};
use vmmap::{Pid, Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

use crate::{
    utils::{bytes_to_usize, select_module, wrap_add},
    Result, Spinner,
};

const PTRS: &str = "PTRS";
const MAPS: &str = "MAPS";
const DATA: &str = "DATA";

#[derive(FromArgs)]
#[argh(description = "Commands.")]
pub struct Commands {
    #[argh(subcommand)]
    pub cmds: CommandEnum,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CommandEnum {
    CreatePointerMap(SubCommandCPM),
    CalcPointerPath(SubCommandCPP),
    ShowPointerPath(SubCommandSPP),
    ShowPointerPathValue(SubCommandSPV),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "cpm", description = "Create Pointer Map.")]
pub struct SubCommandCPM {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "cpp", description = "Calculate Pointer Path.")]
pub struct SubCommandCPP {
    #[argh(option, description = "target address")]
    pub target: Target,
    #[argh(option, description = "dir")]
    pub dir: PathBuf,
    #[argh(option, default = "7", description = "depth")]
    pub depth: usize,
    #[argh(option, default = "Offset((0, 600))", description = "offset")]
    pub offset: Offset,
}

pub struct Target(Address);

impl FromArgValue for Target {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let value = value.trim_start_matches("0x");
        let address = Address::from_str_radix(value, 16).map_err(|e| e.to_string())?;
        Ok(Self(address))
    }
}

impl Deref for Target {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl Deref for Offset {
    type Target = (usize, usize);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(FromArgs)]
#[argh(subcommand, name = "spp", description = "View `CPP` result file")]
pub struct SubCommandSPP {
    #[argh(option, description = "dir")]
    pub dir: PathBuf,

    #[argh(option, default = "30", short = 'n', description = "ppecify the number of data to view")]
    pub num: usize,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "spv", description = "Get the address pointed to by the pointer path.")]
pub struct SubCommandSPV {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,
    #[argh(option, description = "pointer path")]
    pub path: String,
}

impl SubCommandCPP {
    pub fn init(self) -> Result<()> {
        let SubCommandCPP { target, dir, depth, offset } = self;
        let m_read = BufReader::with_capacity(MAX_BUF_SIZE, File::open(dir.join(MAPS))?);
        let maps: Vec<(usize, usize, PathBuf)> = ptrsx_decode_maps(m_read)?;
        let select = select_module(maps.clone())?;

        let mut spinner = Spinner::start("load ptrs cache...");

        let p_read = BufReader::with_capacity(MAX_BUF_SIZE, File::open(dir.join(PTRS))?);

        let dir = PathBuf::from(format!("{:#x}", *target));
        fs::create_dir(&dir)?;

        let mut m_out = BufWriter::with_capacity(
            MAX_BUF_SIZE,
            OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(dir.join(MAPS))?,
        );

        let mut p_out = BufWriter::with_capacity(
            MAX_BUF_SIZE,
            OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(dir.join(DATA))?,
        );

        let params = PathFindParams {
            target: *target,
            depth,
            offset: *offset,
            pout: &mut p_out,
            mout: &mut m_out,
            maps,
            filter: Some(Filter::Range(select)),
        };
        let engine = ptrsx_init_engine(p_read, params)?;

        spinner.stop("load ptrs cache ok");
        let mut spinner = Spinner::start("start calc ptr path...");
        engine.find_pointer_path()?;
        spinner.stop("cacl ptr path ok");
        Ok(())
    }
}

impl SubCommandCPM {
    pub fn init(self) -> Result<()> {
        let mut spinner = Spinner::start("create ptrs cache...");

        let SubCommandCPM { pid } = self;

        let proc = Process::open(pid)?;
        let app_name = proc.app_path().file_name().map(Path::new).unwrap();
        fs::create_dir(app_name)?;

        let p_path = app_name.join(PTRS);
        let m_path = app_name.join(MAPS);

        let p_out = BufWriter::with_capacity(
            MAX_BUF_SIZE,
            OpenOptions::new().write(true).append(true).create(true).open(p_path)?,
        );
        let m_out = BufWriter::with_capacity(
            MAX_BUF_SIZE,
            OpenOptions::new().write(true).append(true).create(true).open(m_path)?,
        );

        ptrsx_create_pointer_map(proc, p_out, m_out)?;

        spinner.stop("create ptrs cache ok");
        Ok(())
    }
}

impl SubCommandSPP {
    pub fn init(self) -> Result<()> {
        let SubCommandSPP { dir, num: _ } = self;

        let binding = std::fs::read(dir.join(DATA))?;
        let (size, data) = binding.split_at(8);
        let size = usize::from_le_bytes(size.try_into().unwrap());
        let mf = File::open(dir.join(MAPS))?;
        let maps: Vec<(usize, usize, PathBuf)> = ptrsx_decode_maps(mf)?;
        let maps = crate::utils::merge_bases(maps);

        let mut buffer = BufWriter::new(std::io::stdout());
        for bin in data.chunks(size) {
            let (off, path) = parse_line(bin).ok_or("err")?;
            let ptr = path.map(|s| s.to_string()).collect::<Vec<_>>().join("->");
            for (start, end, path) in maps.iter() {
                if (start..end).contains(&&off) {
                    let name = path.file_name().unwrap().to_string_lossy();
                    writeln!(buffer, "{name}+{:#x}->{ptr}", off - start)?;
                }
            }
        }

        Ok(())
    }
}

#[inline(always)]
pub fn parse_line(bin: &[u8]) -> Option<(Address, impl Iterator<Item = i16> + '_)> {
    let line = bin.rsplitn(2, |&n| n == 101).nth(1)?;
    let (off, path) = line.split_at(8);
    let off = Address::from_le_bytes(off.try_into().ok()?);
    let path = path.chunks(2).rev().map(|x| i16::from_le_bytes(x.try_into().unwrap()));

    Some((off, path))
}

impl SubCommandSPV {
    pub fn init(self) -> Result<()> {
        let SubCommandSPV { pid, path } = self;
        let proc = Process::open(pid)?;
        let (name, off, offv, last) = parse_path(&path).ok_or("err")?;
        let mut address = proc
            .get_maps()
            .filter(|m| m.is_read() && m.path().is_some())
            .find(|m| m.path().map_or(false, |f| f.file_name().map_or(false, |n| n.eq(name))))
            .map(|m| m.start() + off)
            .ok_or("find modules error")
            .unwrap();

        let mut buf = vec![0; 8];

        for off in offv {
            proc.read_at(wrap_add(address, off).ok_or("err")?, &mut buf)?;
            address = bytes_to_usize(buf.as_mut_slice())?;
        }

        println!("{:#x}", wrap_add(address, last).ok_or("err")?);

        Ok(())
    }
}

#[inline(always)]
pub fn parse_path(path: &str) -> Option<(&str, usize, Vec<i16>, i16)> {
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
