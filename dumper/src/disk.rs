use std::{fs::OpenOptions, io::BufWriter};

use consts::MAX_BUF_SIZE;
use vmmap::{Process, ProcessInfo};

use super::cmd::SubCommandDisk;
use crate::a::create_pointer_map_helper;

impl SubCommandDisk {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandDisk { pid, dir } = self;
        let proc = Process::open(pid)?;
        let name = proc
            .app_path()
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or("get app_name error")?;
        let path = dir.with_file_name(format!("{name}-{pid}")).with_extension("dump");

        let out = BufWriter::with_capacity(
            MAX_BUF_SIZE,
            OpenOptions::new().write(true).append(true).create(true).open(path)?,
        );

        create_pointer_map_helper(proc, out)?;

        Ok(())
    }
}
