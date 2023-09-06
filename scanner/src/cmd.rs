use std::path::PathBuf;

use argh::{FromArgValue, FromArgs};

pub struct Address(pub usize);

impl FromArgValue for Address {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let value = value.trim_start_matches("0x");
        let address = usize::from_str_radix(value, 16).map_err(|e| e.to_string())?;
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
#[argh(description = "PointerSearch-X")]
pub struct Commands {
    #[argh(subcommand)]
    pub cmds: CommandEnum,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CommandEnum {
    Scan1(SubCommandScan1),
    Scan2(SubCommandScan2),
    Diff(SubCommandDiff),
}

#[derive(FromArgs)]
#[argh(
    subcommand,
    name = "s1",
    description = "Scan mode 1, select some modules to set as base addresses."
)]
pub struct SubCommandScan1 {
    #[argh(option, short = 'f', description = "ptrs file path")]
    pub file: PathBuf,
    #[argh(option, short = 't', description = "target address")]
    pub target: Address,
    #[argh(option, default = "7", short = 'd', description = "depth")]
    pub depth: usize,
    #[argh(option, default = "Offset((0, 600))", short = 'o', description = "offset")]
    pub offset: Offset,
    #[argh(option, default = "3", short = 'n', description = "node")]
    pub node: usize,
    #[argh(option, description = "out dir")]
    pub dir: Option<PathBuf>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "s2", description = "Scan mode 2, set address as the base address.")]
pub struct SubCommandScan2 {
    #[argh(option, short = 'f', description = "ptrs file path")]
    pub file: PathBuf,
    #[argh(option, short = 's', description = "start address")]
    pub start: Address,
    #[argh(option, short = 't', description = "target address")]
    pub target: Address,
    #[argh(option, default = "7", short = 'd', description = "depth")]
    pub depth: usize,
    #[argh(option, default = "Offset((0, 600))", short = 'o', description = "offset")]
    pub offset: Offset,
    #[argh(option, default = "3", short = 'n', description = "node")]
    pub node: usize,
    #[argh(option, description = "out dir")]
    pub dir: Option<PathBuf>,
}

#[derive(FromArgs)]
#[argh(
    subcommand,
    name = "diff",
    description = "Compare and get the intersecting parts of two .scandata files."
)]
pub struct SubCommandDiff {
    #[argh(option, description = "file1 path")]
    pub f1: PathBuf,
    #[argh(option, description = "file2 path")]
    pub f2: PathBuf,
    #[argh(option, description = "out file name")]
    pub out: Option<PathBuf>,
}
