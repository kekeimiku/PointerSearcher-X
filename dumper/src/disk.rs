use std::{fs::OpenOptions, io::BufWriter};

use ptrsx::{c64::default_dump_ptr, DEFAULT_BUF_SIZE};
use vmmap::vmmap64::{Process, ProcessInfo};

use crate::{cmd::SubCommandDisk, utils::Spinner};

impl SubCommandDisk {
    pub fn init(self) -> Result<(), super::error::Error> {
        let SubCommandDisk { pid, out } = self;
        let proc = Process::open(pid)?;
        let name = proc
            .app_path()
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or("get app_name error")?;
        let out = match out {
            Some(file) => OpenOptions::new().write(true).append(true).create(true).open(file),
            None => OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(format!("{name}-{pid}.dump")),
        }?;
        let mut spinner = Spinner::start("Start dump pointers...");
        let mut writer = BufWriter::with_capacity(DEFAULT_BUF_SIZE, out);
        default_dump_ptr(&proc, &mut writer)?;
        spinner.stop("Dump completed.");

        Ok(())
    }
}
