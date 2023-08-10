use std::{fs::OpenOptions, io::BufWriter, path::Path};

use ptrsx::{Params, PtrsxScanner};

use super::{select_base_module, Error, Spinner, SubCommandScan};

impl SubCommandScan {
    pub fn init(self) -> Result<(), Error> {
        let SubCommandScan { ref file, target, depth, offset, node, dir } = self;

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
            .try_for_each(|(base, name, points)| {
                let name = Path::new(name)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .expect("get region name error");
                let file = dir.join(name).with_extension("scandata");
                let file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create_new(true)
                    .open(file)?;
                let params = Params {
                    base,
                    depth,
                    target: target.0,
                    node,
                    range: offset.0,
                    points: &points,
                    writer: &mut BufWriter::new(file),
                };
                ptrsx.scan(params)
            })?;
        spinner.stop("Pointer chain is scanned.");

        Ok(())
    }
}
