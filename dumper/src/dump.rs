use std::{fs::OpenOptions, io::BufWriter};

use ptrsx::{PtrsxScanner, DEFAULT_BUF_SIZE};
use vmmap::{Process, ProcessInfo};

use super::{DumpCommand, Error, Spinner};

impl DumpCommand {
    pub fn init(self) -> Result<(), Error> {
        let DumpCommand { pid, file, align } = self;
        let proc = Process::open(pid)?;

        let out = match file {
            Some(file) => OpenOptions::new().write(true).append(true).create(true).open(file),
            None => {
                let name = proc
                    .app_path()
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("unknown");

                OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create_new(true)
                    .open(format!("{name}-{pid}.dump"))
            }
        }?;
        let mut spinner = Spinner::start("Start dump pointers...");
        let ptrsx = PtrsxScanner::default();
        let mut writer = BufWriter::with_capacity(DEFAULT_BUF_SIZE, out);
        ptrsx.create_pointer_map_file(&mut writer, pid, align)?;
        spinner.stop("Dump completed.");

        Ok(())
    }
}
