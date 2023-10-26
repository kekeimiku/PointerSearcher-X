use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
};

use super::{Error, SubCommandDiff};

impl SubCommandDiff {
    pub fn init(self) -> Result<(), Error> {
        let SubCommandDiff { f1, f2, out } = self;

        let h1 = fs::read_to_string(f1)?;
        let h2 = fs::read_to_string(f2)?;
        let h1 = h1.lines().collect::<HashSet<_>>();
        let h2 = h2.lines().collect::<HashSet<_>>();

        let out: Box<dyn Write> = match out {
            Some(file) => Box::new(OpenOptions::new().write(true).append(true).create(true).open(file)?) as _,
            None => Box::new(io::stdout()) as _,
        };
        let mut out = BufWriter::new(out);

        Ok(h1.intersection(&h2).try_for_each(|s| writeln!(out, "{s}"))?)
    }
}
