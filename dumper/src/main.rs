use dumper::{CommandEnum, Commands};

fn main() {
    if let Err(err) = match argh::from_env::<Commands>().cmds {
        CommandEnum::WithDisk(this) => this.init(),
        CommandEnum::TestPtrs(this) => this.init(),
    } {
        eprintln!("{err}")
    }
}
