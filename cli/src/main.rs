use cli::cmd::{CommandEnum, Commands};

fn main() {
    let args: Commands = argh::from_env();
    match args.cmds {
        CommandEnum::CreatePointerMap(args) => args.init().unwrap(),
        CommandEnum::CalcPointerPath(args) => args.init().unwrap(),
        CommandEnum::ShowPointerPath(args) => args.init().unwrap(),
        CommandEnum::ShowPointerPathValue(args) => args.init().unwrap(),
    }
}
