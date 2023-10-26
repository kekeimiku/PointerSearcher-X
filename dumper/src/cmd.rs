use std::path::PathBuf;

use argh::FromArgs;
use vmmap::Pid;

#[derive(FromArgs)]
#[argh(description = "Commands.")]
pub struct Commands {
    #[argh(subcommand)]
    pub cmds: CommandEnum,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CommandEnum {
    DumpProcess(DumpCommand),
    PointerChain(ChainCommand),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "disk", description = "dump process pointer to disk")]
pub struct DumpCommand {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,

    #[argh(option, short = 'f', description = "out filename")]
    pub file: Option<PathBuf>,

    #[argh(option, default = "true", description = "pointer align, default true")]
    pub align: bool,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "test", description = "test pointer chain")]
pub struct ChainCommand {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,

    #[argh(option, description = "pointer chain")]
    pub chain: String,

    #[argh(option, short = 'n', description = "show bytes")]
    pub num: Option<usize>,
}
