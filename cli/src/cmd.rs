use std::path::PathBuf;

use argh::FromArgs;
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
    CaltPointerPath(SubCommandCPP),
    DiffPointerPath(SubCommandDIFF),
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
    #[argh(option, short = 'f', description = "file path, you can specify multiple")]
    pub file: Vec<PathBuf>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "spp", description = "Show Pointer Path.")]
pub struct SubCommandSPP {
    #[argh(option, short = 'f', description = "file path")]
    pub file: PathBuf,

    #[argh(option, default = "30", short = 'n', description = "ppecify the number of data to view")]
    pub num: usize,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "diff", description = "Diff Pointer Path File.")]
pub struct SubCommandDIFF {
    #[argh(option, short = 'f', description = "file path, you can specify multiple")]
    pub file: Vec<PathBuf>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "spv", description = "Get the address pointed to by the pointer path.")]
pub struct SubCommandSPV {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,
    #[argh(option, description = "pointer path")]
    pub path: String,
}
