use std::{fs::OpenOptions, io::BufWriter};

use ptrsx::{consts::MAX_BUF_SIZE, dumper::PtrsXDumper};

use super::cmd::SubCommandDisk;

impl SubCommandDisk {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandDisk { pid, out } = self;
        let dumper = PtrsXDumper::init(pid)?;

        let out = match out {
            Some(file) => OpenOptions::new().write(true).append(true).create_new(true).open(file),
            None => OpenOptions::new()
                .write(true)
                .append(true)
                .create_new(true)
                .open(format!("{pid}.dump")),
        }?;
        let mut out = BufWriter::with_capacity(MAX_BUF_SIZE, out);

        dumper.create_pointer_map_helper(&mut out)?;

        Ok(())
    }
}
