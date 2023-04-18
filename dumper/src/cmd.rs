use std::{net::SocketAddr, path::PathBuf};

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
    WithNet(SubCommandNet),
    WithDisk(SubCommandDisk),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "disk", description = "use disk")]
pub struct SubCommandDisk {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,

    #[argh(option, default = "PathBuf::new()", description = "out dir path")]
    pub dir: PathBuf,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "net", description = "use net")]
pub struct SubCommandNet {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,

    #[argh(option, description = "out url address")]
    pub url: SocketAddr,
}
