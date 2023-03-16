use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum CommandEnum {
    PointerScanner(SubCommandPs),
    DiffPointerMaps(SubCommandDm),
    ShowPointerMaps(SubCommandSm),
    ViewPointerPath(SubCommandSp),
    MemoryScanner(SumCommandMs),
    PatchMemory(SubCommandPc),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ps", description = "pointer scanner")]
pub struct SubCommandPs {
    /// process id, type int32
    #[argh(option, short = 'p')]
    pub pid: i32,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "dm", description = "diff pointer maps")]
pub struct SubCommandDm {
    /// file1
    #[argh(option)]
    pub f1: String,
    /// file2
    #[argh(option)]
    pub f2: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "sm", description = "show pointer maps")]
pub struct SubCommandSm {
    /// file
    #[argh(option)]
    pub f: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "sp", description = "show pointer path value")]
pub struct SubCommandSp {
    /// process id, type int32
    #[argh(option, short = 'p')]
    pub pid: i32,
    /// pointer path
    #[argh(option, short = 't')]
    pub target: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ms", description = "memory scanner")]
pub struct SumCommandMs {
    /// process id
    #[argh(option, short = 'p')]
    pub pid: i32,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "pc", description = "patch address")]
pub struct SubCommandPc {
    /// process id
    #[argh(option, short = 'p')]
    pub pid: i32,

    /// target address
    #[argh(option)]
    pub addr: String,

    /// target value
    #[argh(option)]
    pub value: i32,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
pub struct Commands {
    #[argh(subcommand)]
    pub nested: CommandEnum,
}
