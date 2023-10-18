use scanner::{CommandEnum, Commands};

fn main() {
    if let Err(err) = match argh::from_env::<Commands>().cmds {
        CommandEnum::Scan(this) => this.init(),
        CommandEnum::Diff(this) => this.init(),
    } {
        eprintln!("\n\x1b[31m error: {err} \x1b[0m")
    }
}
