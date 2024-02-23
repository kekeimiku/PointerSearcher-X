use std::path::PathBuf;

use argh::{FromArgValue, FromArgs};
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
    TestChain(TestChainCommand),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "disk", description = "dump process pointer to disk")]
pub struct DumpCommand {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,

    #[argh(option, description = "modules info out filename")]
    pub info: Option<PathBuf>,

    #[argh(option, description = "binary data out filename")]
    pub bin: Option<PathBuf>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "test", description = "test pointer chain")]
pub struct TestChainCommand {
    #[argh(option, short = 'p', description = "process id")]
    pub pid: Pid,

    #[argh(option, description = "pointer chain")]
    pub chain: String,

    #[argh(option, short = 'w', description = "write bytes")]
    pub write: Option<WVecU8>,

    #[argh(option, short = 'r', description = "read bytes")]
    pub read: Option<usize>,
}

pub struct WVecU8(pub Vec<u8>);

impl FromArgValue for WVecU8 {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let parts = value.split(['[', ']', ',', ' ']).filter(|s| !s.is_empty());
        let bytes = parts
            .map(|s| u8::from_str_radix(s.trim().trim_start_matches("0x"), 16))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| "parse bytes failed")?;
        Ok(Self(bytes))
    }
}
