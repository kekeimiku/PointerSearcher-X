use dumper::{CommandEnum, Commands};

fn main() {
    if let Err(err) = match argh::from_env::<Commands>().cmds {
        CommandEnum::DumpProcess(this) => this.init(),
        CommandEnum::PointerChain(this) => this.init(),
    } {
        eprintln!("\n\x1b[31m error: {err} \x1b[0m")
    }
}
