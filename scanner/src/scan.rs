use std::{fs::OpenOptions, io::BufWriter, path::Path};

use ptrsx::{Params, PtrsxScanner};

use super::{select_base_module, Address, Error, Offset, Spinner, SubCommandScan1, SubCommandScan2};

impl SubCommandScan1 {
    pub fn init(self) -> Result<(), Error> {
        let Self {
            ref file,
            target: Address(target),
            depth,
            offset: Offset(offset),
            node,
            dir,
        } = self;

        if depth <= node {
            println!("Error: depth must be greater than node. current depth({depth}), node({node}).")
        }

        let mut spinner = Spinner::start("Start loading cache...");
        let ptrsx = PtrsxScanner::load_with_file(file)?;
        spinner.stop("cache loaded.");

        let pages = select_base_module(ptrsx.pages())?;
        let mut spinner = Spinner::start("Start creating pointer maps...");
        spinner.stop("Pointer map is created.");

        let dir = dir.unwrap_or_default();

        let mut spinner = Spinner::start("Start scanning pointer chain...");
        pages
            .iter()
            .map(|m| (m.start, m.path, ptrsx.range_address(m).collect::<Vec<_>>()))
            .try_for_each(|(base, name, ref points)| {
                let name = Path::new(name)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .expect("get region name error");
                let file = dir.join(format!("{name}.scandata"));
                let file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create_new(true)
                    .open(file)?;
                #[rustfmt::skip]
                let params = Params {
                    base, depth, target, node, offset, points,
                    writer: &mut BufWriter::new(file),
                };
                ptrsx.scan(params)
            })?;
        spinner.stop("Pointer chain is scanned.");

        Ok(())
    }
}

impl SubCommandScan2 {
    pub fn init(self) -> Result<(), Error> {
        let Self {
            ref file,
            start: Address(start),
            target: Address(target),
            depth,
            offset: Offset(offset),
            node,
            dir,
        } = self;
        if depth <= node {
            println!("Error: depth must be greater than node. current depth({depth}), node({node}).")
        }

        let mut spinner = Spinner::start("Start loading cache...");
        let ptrsx = PtrsxScanner::load_with_file(file)?;
        spinner.stop("cache loaded.");
        let dir = dir.unwrap_or_default();

        let mut spinner = Spinner::start("Start scanning pointer chain...");
        let file = dir.join(format!("{:#x}.scandata", start));
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create_new(true)
            .open(file)?;
        #[rustfmt::skip]
        let params = Params {
            base: 0,
            depth, target, node, offset,
            points: &[start],
            writer: &mut BufWriter::new(file),
        };

        ptrsx.scan(params)?;

        spinner.stop("Pointer chain is scanned.");

        Ok(())
    }
}
