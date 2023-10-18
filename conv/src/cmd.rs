use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "Conv")]
pub struct Commands {
    #[argh(subcommand)]
    pub cmds: CommandEnum,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CommandEnum {
    Pince(SubCommandPince),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "pince", description = "conv pince .pct file.")]
pub struct SubCommandPince {
    #[argh(option, description = "scandata file path")]
    pub scandata: PathBuf,
    #[argh(option, description = "pct file path")]
    pub pct: Option<PathBuf>,
}
