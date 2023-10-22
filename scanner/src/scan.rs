use std::{fs::OpenOptions, io::BufWriter, path::Path};

use ptrsx::{Params, PtrsxScanner};

use super::{select_base_module, Address, AddressList, Error, Offset, Spinner, SubCommandScan1, SubCommandScan2};

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
        let mut ptrsx = PtrsxScanner::default();
        ptrsx.load_pointer_map_file(file)?;
        spinner.stop("cache loaded.");

        let pages = select_base_module(&ptrsx.modules)?;
        let mut spinner = Spinner::start("Start creating pointer maps...");
        spinner.stop("Pointer map is created.");

        let dir = dir.unwrap_or_default();

        let mut spinner = Spinner::start("Start scanning pointer chain...");
        pages.iter().try_for_each(|module| {
            let name = Path::new(&module.name)
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
                    depth, target, node, offset,
                    writer: &mut BufWriter::new(file),
                };
            ptrsx.scanner_with_range(module.start..module.end, params)
        })?;
        spinner.stop("Pointer chain is scanned.");

        Ok(())
    }
}

impl SubCommandScan2 {
    pub fn init(self) -> Result<(), Error> {
        let Self {
            ref file,
            list: AddressList(ref list),
            target: Address(target),
            depth,
            offset: Offset(offset),
            node,
            dir,
        } = self;
        if depth <= node {
            return Err(format!("Error: depth must be greater than node. current depth({depth}), node({node}).").into());
        }
        if depth > 32 {
            return Err(format!("Error: MAX_DEPTH 32, current({depth})").into());
        }

        let mut spinner = Spinner::start("Start loading cache...");
        let mut ptrsx = PtrsxScanner::default();
        ptrsx.load_pointer_map_file(file)?;
        spinner.stop("cache loaded.");
        let dir = dir.unwrap_or_default();

        let mut spinner = Spinner::start("Start scanning pointer chain...");
        let file = dir.join(format!("{target:#x}.scandata"));
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create_new(true)
            .open(file)?;
        #[rustfmt::skip]
        let params = Params {
            depth, target, node, offset,
            writer: &mut BufWriter::new(file),
        };

        ptrsx.scanner_with_address(list, params)?;

        spinner.stop("Pointer chain is scanned.");

        Ok(())
    }
}
