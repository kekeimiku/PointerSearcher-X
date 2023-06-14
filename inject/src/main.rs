use std::path::PathBuf;

use argh::FromArgs;
use inject::inject;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmds = argh::from_env::<Commands>();
    inject(cmds.pid, cmds.path)?;
    println!("Injected successfully");
    Ok(())
}

#[derive(FromArgs)]
#[argh(description = "inject.")]
pub struct Commands {
    #[argh(option, description = "pid")]
    pub pid: i32,

    #[argh(option, description = "path")]
    pub path: PathBuf,
}
