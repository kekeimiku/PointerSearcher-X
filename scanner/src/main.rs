use scanner::{CommandEnum, Commands};

fn main() -> Result<(), scanner::Error> {
    match argh::from_env::<Commands>().cmds {
        CommandEnum::Scanner(this) => this.init(),
        CommandEnum::Diff(this) => this.init(),
    }
}
