use dumper::cmd::{CommandEnum, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match argh::from_env::<Commands>().cmds {
        CommandEnum::WithNet(_) => todo!(),
        CommandEnum::WithDisk(this) => this.init(),
        CommandEnum::TestPtrs(this) => this.init(),
    }
}
