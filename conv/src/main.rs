use conv::{CommandEnum, Commands};

fn main() {
    if let Err(err) = match argh::from_env::<Commands>().cmds {
        CommandEnum::Pince(this) => this.init(),
    } {
        eprintln!("\x1b[31m error: {err} \x1b[0m")
    }
}
