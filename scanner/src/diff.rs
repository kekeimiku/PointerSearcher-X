use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
};

use super::cmd::SubCommandDiff;

impl SubCommandDiff {
    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let SubCommandDiff { f1, f2, out } = self;

        let h1 = unsafe { String::from_utf8_unchecked(fs::read(f1)?) };
        let h1 = h1.lines().collect::<HashSet<_>>();

        let h2 = unsafe { String::from_utf8_unchecked(fs::read(f2)?) };
        let h2 = h2.lines().collect::<HashSet<_>>();

        let out: Box<dyn Write> = match out {
            Some(file) => Box::new(OpenOptions::new().write(true).append(true).create(true).open(file)?) as _,
            None => Box::new(io::stdout()) as _,
        };
        let mut out = BufWriter::new(out);

        Ok(h1.intersection(&h2).try_for_each(|s| writeln!(out, "{s}"))?)
    }
}
