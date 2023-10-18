use scanner::{CommandEnum, Commands};

fn main() {
    if let Err(err) = match argh::from_env::<Commands>().cmds {
        CommandEnum::Scan1(this) => this.init(),
        CommandEnum::Diff(this) => this.init(),
        CommandEnum::Scan2(this) => this.init(),
    } {
        eprintln!("{err}")
    }
}
