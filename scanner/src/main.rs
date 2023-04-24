#[cfg(not(all(unix, target_pointer_width = "64", target_endian = "little")))]
panic!("Not support");

use ptrsx_scanner::cmd::{CommandEnum, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match argh::from_env::<Commands>().cmds {
        CommandEnum::Scanner(args) => args.init(),
        CommandEnum::Convert(args) => args.init(),
        CommandEnum::Diff(args) => args.init(),
    }
}
