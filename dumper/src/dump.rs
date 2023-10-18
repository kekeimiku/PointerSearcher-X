use std::{fs::OpenOptions, path::PathBuf};

use ptrsx::PtrsxScanner;

use super::{DumpCommand, Error, Spinner};

impl DumpCommand {
    pub fn init(self) -> Result<(), Error> {
        let DumpCommand { pid, info, bin, align } = self;
        let info = info.unwrap_or_else(|| PathBuf::from(format!("{pid}.info.txt")));
        let bin = bin.unwrap_or_else(|| PathBuf::from(format!("{pid}.bin")));
        let mut spinner = Spinner::start("start dump pointers...");
        let ptrsx = PtrsxScanner::default();

        let info = OpenOptions::new().append(true).create_new(true).open(info)?;
        let bin = OpenOptions::new().append(true).create_new(true).open(bin)?;
        ptrsx.create_pointer_map(pid, align, info, bin)?;
        spinner.stop("dump is finished.");

        Ok(())
    }
}
