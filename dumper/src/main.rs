use dumper::cmd::{CommandEnum, Commands};

fn main() -> Result<(), dumper::error::Error> {
    match argh::from_env::<Commands>().cmds {
        CommandEnum::WithDisk(this) => this.init(),
        CommandEnum::TestPtrs(this) => this.init(),
    }
}
