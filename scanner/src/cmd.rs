use std::path::PathBuf;

use argh::{FromArgValue, FromArgs};

pub struct Target(pub usize);

impl FromArgValue for Target {
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
#[argh(description = "Commands.")]
pub struct Commands {
    #[argh(subcommand)]
    pub cmds: CommandEnum,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CommandEnum {
    Scanner(SubCommandScan),
    Diff(SubCommandDiff),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "scan", description = "scanner")]
pub struct SubCommandScan {
    #[argh(option, short = 'f', description = "ptrs file path")]
    pub file: PathBuf,
    #[argh(option, short = 't', description = "target address")]
    pub target: Target,
    #[argh(option, default = "7", short = 'd', description = "depth")]
    pub depth: usize,
    #[argh(option, default = "Offset((0, 600))", short = 'o', description = "offset")]
    pub offset: Offset,
    #[argh(option, description = "out dir")]
    pub dir: Option<PathBuf>,
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
