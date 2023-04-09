use cli::{CommandEnum, Commands};

fn main() -> cli::Result<()> {
    let args: Commands = argh::from_env();
    match args.cmds {
        CommandEnum::CreatePointerMap(args) => args.init(),
        CommandEnum::CalcPointerPath(args) => args.init(),
        CommandEnum::ShowPointerPath(args) => args.init(),
        CommandEnum::ShowPointerPathValue(args) => args.init(),
    }
}
