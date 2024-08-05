mod cmd;
mod error;
mod ptrscan;
mod ptrscan_bindgen;
mod utils;

use cmd::PointerSearch;

fn main() {
    let ptrscan: PointerSearch = argh::from_env();
    if let Err(err) = match ptrscan.cmds {
        cmd::TopCommandEnum::ListModules(this) => this.init(),
        cmd::TopCommandEnum::CreatePointerMap(this) => this.init(),
        cmd::TopCommandEnum::ScanPointerChain(this) => match this.cmds {
            cmd::ScanCommandEnum::UsePointerMapFile(this) => this.init(),
            cmd::ScanCommandEnum::UseProcessID(this) => this.init(),
        },
        cmd::TopCommandEnum::CmpPointerChain(this) => this.init(),
        cmd::TopCommandEnum::TestPointerChain(this) => this.init(),
    } {
        eprintln!("\n\x1b[31m error: {err} \x1b[0m")
    }
}
