use dumper::{CommandEnum, Commands};

fn main() -> Result<(), dumper::Error> {
    match argh::from_env::<Commands>().cmds {
        CommandEnum::WithDisk(this) => this.init(),
        CommandEnum::TestPtrs(this) => this.init(),
    }
}
