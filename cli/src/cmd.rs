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
    pointer_path::{ptrsx_init_engine, PathFindParams},
};
use vmmap::{Pid, Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

use crate::{
    error::Result,
    spinner::Spinner,
    utils::{bytes_to_usize, select_module, wrap_add},
};

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
    #[argh(option, description = "pointer file path")]
    pub pf: PathBuf,
    #[argh(option, description = "maps file path")]
    pub mf: PathBuf,
    #[argh(option, default = "7", description = "depth")]
    pub depth: usize,
    #[argh(option, default = "Offset((0, 800))", description = "offset")]
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
    #[argh(option, description = "result file path")]
    pub rf: PathBuf,

    #[argh(option, description = "maps file path")]
    pub mf: PathBuf,

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
        let SubCommandCPP { target, pf, mf, depth, offset } = self;
        let m_read = BufReader::with_capacity(MAX_BUF_SIZE, File::open(mf)?);
        let maps: Vec<(usize, usize, PathBuf)> = ptrsx_decode_maps(m_read)?;

        let filter = select_module(maps)?
            .into_iter()
            .map(|(start, end, _)| (start..end))
            .collect();

        let mut spinner = Spinner::start("load ptrs cache...");

        let p_read = BufReader::with_capacity(MAX_BUF_SIZE, File::open(pf)?);
        let size = depth * 2 + 9;
        let out = Path::new("./")
            .with_file_name(target.to_string())
            .with_extension(size.to_string());
        let mut out =
            BufWriter::with_capacity(MAX_BUF_SIZE, OpenOptions::new().write(true).append(true).create(true).open(out)?);

        let params = PathFindParams {
            target: *target,
            depth,
            offset: *offset,
            out: &mut out,
            filter: Some(filter),
            startpoints: None,
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
        let app_name = proc.app_path().file_name().unwrap();
        let p_path = Path::new("./").with_file_name(app_name).with_extension("ptrs");
        let m_path = Path::new("./").with_file_name(app_name).with_extension("maps");

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
        let SubCommandSPP { rf, mf, num: _ } = self;
        let size = rf
            .extension()
            .and_then(|s| s.to_str().and_then(|s| s.parse::<usize>().ok()))
            .unwrap();

        let data = fs::read(rf)?;
        let mf = File::open(mf)?;
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
