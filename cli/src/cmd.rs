use std::{ops::Deref, path::PathBuf};

use argh::{FromArgValue, FromArgs};
use vmmap::Pid;

#[derive(FromArgs)]
#[argh(description = "Top-level command.")]
pub struct Commands {
    #[argh(subcommand)]
    pub nested: CommandEnum,
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
    pub target: Address,
    #[argh(option, description = "pointer file path")]
    pub pf: PathBuf,
    #[argh(option, description = "maps file path")]
    pub mf: PathBuf,
    #[argh(option, default = "7", description = "depth")]
    pub depth: usize,
    #[argh(option, default = "Offset((0, 800))", description = "offset")]
    pub offset: Offset,
}

pub struct Address(crate::consts::Address);

impl FromArgValue for Address {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let value = value.trim_start_matches("0x");
        let address = crate::consts::Address::from_str_radix(value, 16).map_err(|e| e.to_string())?;
        Ok(Self(address))
    }
}

impl Deref for Address {
    type Target = crate::consts::Address;

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
