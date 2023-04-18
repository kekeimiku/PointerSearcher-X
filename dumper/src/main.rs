use dumper::cmd::{CommandEnum, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match argh::from_env::<Commands>().cmds {
        CommandEnum::WithDisk(args) => args.init(),
        CommandEnum::WithNet(args) => args.init(),
    }
}
