use std::{fs::OpenOptions, io::BufWriter};

use ptrsx::{PtrsxScanner, DEFAULT_BUF_SIZE};
use vmmap::{Process, ProcessInfo};

use super::{Error, Spinner, SubCommandDisk};

impl SubCommandDisk {
    pub fn init(self) -> Result<(), Error> {
        let SubCommandDisk { pid, out, align } = self;
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
                .create_new(true)
                .open(format!("{name}-{pid}.dump")),
        }?;
        let mut spinner = Spinner::start("Start dump pointers...");
        let ptrsx = PtrsxScanner::default();
        let mut writer = BufWriter::with_capacity(DEFAULT_BUF_SIZE, out);
        ptrsx.create_pointer_map_file(&mut writer, pid, align)?;
        spinner.stop("Dump completed.");

        Ok(())
    }
}
