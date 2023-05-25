use std::{
    collections::HashSet,
    fs,
    fs::OpenOptions,
    io,
    io::{BufWriter, Write},
    path::PathBuf,
};

use argh::{FromArgValue, FromArgs};
use ptrsx::{
    consts::{Address, MAX_BUF_SIZE},
    dumper::merge_bases,
    scanner::ScannerParm,
};

use crate::{
    b::convert_bin_to_txt,
    utils::{select_module, Spinner},
};

pub struct Target(pub Address);

impl FromArgValue for Target {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let value = value.trim_start_matches("0x");
        let address = Address::from_str_radix(value, 16).map_err(|e| e.to_string())?;
        Ok(Self(address))
    }
}

pub struct Offset(pub (usize, usize));

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
    #[argh(subcommand)]
    pub cmds: CommandEnum,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CommandEnum {
    Scanner(SubCommandScan),
    Convert(SubCommandConvert),
    Diff(SubCommandDiff),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "scan", description = "scanner")]
pub struct SubCommandScan {
    #[argh(option, short = 'f', description = "file path")]
    pub file: PathBuf,
    #[argh(option, short = 't', description = "target address")]
    pub target: Target,
    #[argh(option, default = "7", short = 'd', description = "depth")]
    pub depth: usize,
    #[argh(option, default = "Offset((0, 600))", short = 'o', description = "offset")]
    pub offset: Offset,
    #[argh(option, description = "out file")]
    pub out: Option<PathBuf>,
}

impl SubCommandScan {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandScan { file, target, depth, offset, .. } = self;
        let mut spinner = Spinner::start("Start loading cache...");
        let scanner = ptrsx::scanner::PtrsXScanner::init(file)?;
        spinner.stop("cache loaded.");

        let pages = select_module(merge_bases(scanner.pages().to_vec()))?;

        let parm = ScannerParm { target: target.0, depth, offset: offset.0, pages };

        let mut spinner = Spinner::start("Start scanning pointer path...");

        scanner.scanner(parm)?;

        spinner.stop("Pointer path is scanned.");

        Ok(())
    }
}

#[derive(FromArgs)]
#[argh(subcommand, name = "conv", description = "convert bin to txt")]
pub struct SubCommandConvert {
    #[argh(option, short = 'f', description = "file path")]
    pub file: PathBuf,
    #[argh(option, description = "out file name")]
    pub out: Option<PathBuf>,
}

impl SubCommandConvert {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandConvert { file, out } = self;

        let out: Box<dyn Write> = match out {
            Some(file) => Box::new(
                OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create_new(true)
                    .open(file)?,
            ) as _,
            None => Box::new(io::stdout()) as _,
        };
        let out = BufWriter::with_capacity(MAX_BUF_SIZE, out);

        convert_bin_to_txt(file, out)
    }
}

#[derive(FromArgs)]
#[argh(subcommand, name = "diff", description = "diff")]
pub struct SubCommandDiff {
    #[argh(option, description = "file1 path")]
    pub f1: PathBuf,
    #[argh(option, description = "file2 path")]
    pub f2: PathBuf,
    #[argh(option, description = "out file name")]
    pub out: Option<PathBuf>,
}

impl SubCommandDiff {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandDiff { f1, f2, out } = self;

        let h1 = unsafe { String::from_utf8_unchecked(fs::read(f1)?) };
        let h1 = h1.lines().collect::<HashSet<_>>();

        let h2 = unsafe { String::from_utf8_unchecked(fs::read(f2)?) };
        let h2 = h2.lines().collect::<HashSet<_>>();

        let out: Box<dyn Write> = match out {
            Some(file) => Box::new(
                OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create_new(true)
                    .open(file)?,
            ) as _,
            None => Box::new(io::stdout()) as _,
        };
        let mut out = BufWriter::with_capacity(MAX_BUF_SIZE, out);

        Ok(h1.intersection(&h2).try_for_each(|s| writeln!(out, "{s}"))?)
    }
}
