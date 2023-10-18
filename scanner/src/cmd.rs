use std::{collections::BTreeSet, path::PathBuf};

use argh::{FromArgValue, FromArgs};

pub struct AddressList(pub Vec<usize>);

impl FromArgValue for AddressList {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let bset = value
            .split(['-', ',', ';'])
            .filter(|s| !s.is_empty())
            .map(|s| usize::from_str_radix(s.trim_start_matches("0x"), 16))
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(Self(bset.into_iter().collect()))
    }
}

pub struct Range(pub (usize, usize));

impl FromArgValue for Range {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let (lr, ur) = value.split_once(':').ok_or(format!("parse command failed: {value}"))?;
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
    Scan(SubCommandScan),
    Diff(SubCommandDiff),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "scan", description = "Pointer chain scan.")]
pub struct SubCommandScan {
    #[argh(option, description = "binary file path")]
    pub bin: PathBuf,
    #[argh(option, description = "info file path")]
    pub info: PathBuf,
    #[argh(option, short = 'l', description = "target address list")]
    pub list: AddressList,
    #[argh(option, default = "4", short = 'd', description = "depth default 4")]
    pub depth: usize,
    #[argh(option, default = "Range((0, 4000))", short = 'r', description = "range default 0:4000")]
    pub range: Range,
    #[argh(option, default = "1", short = 'n', description = "node default 1")]
    pub node: usize,
    #[argh(option, description = "out dir")]
    pub dir: Option<PathBuf>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "diff", description = "Get the intersection of two .scandata files.")]
pub struct SubCommandDiff {
    #[argh(option, description = "file1 name")]
    pub f1: PathBuf,
    #[argh(option, description = "file2 name")]
    pub f2: PathBuf,
    #[argh(option, description = "out file name")]
    pub out: Option<PathBuf>,
}
