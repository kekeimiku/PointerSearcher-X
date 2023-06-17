use scanner::cmd::{CommandEnum, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match argh::from_env::<Commands>().cmds {
        CommandEnum::Scanner(this) => this.init(),
        CommandEnum::Diff(this) => this.init(),
    }
}
