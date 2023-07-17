use std::path::PathBuf;

use argh::FromArgs;
use inject::inject;

fn main() {
    let cmds = argh::from_env::<Commands>();
    match inject(cmds.pid, cmds.path) {
        Ok(_) => println!("Injected successfully"),
        Err(err) => println!("{err}"),
    };
}

#[derive(FromArgs)]
#[argh(description = "inject.")]
pub struct Commands {
    #[argh(option, description = "pid")]
    pub pid: i32,

    #[argh(option, description = "path")]
    pub path: PathBuf,
}
