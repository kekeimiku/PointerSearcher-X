use std::path::PathBuf;

use ptrsx::PtrsxScanner;
use vmmap::Process;

use super::{DumpCommand, Error, Spinner};

impl DumpCommand {
    pub fn init(self) -> Result<(), Error> {
        let DumpCommand { pid, info, bin } = self;
        let info = info.unwrap_or_else(|| PathBuf::from(format!("{pid}.info.txt")));
        let bin = bin.unwrap_or_else(|| PathBuf::from(format!("{pid}.bin")));
        let mut spinner = Spinner::start("start dump pointers...");
        let ptrsx = PtrsxScanner::default();

        let proc = Process::open(pid)?;
        ptrsx.create_pointer_map(&proc, info, bin)?;
        spinner.stop("dump is finished.");

        Ok(())
    }
}
