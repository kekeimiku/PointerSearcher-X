use std::{fs::OpenOptions, io::BufWriter, path::Path};

use ptrsx::{s64::Params, sc64::PtrsxScanner};

use super::{
    cmd::SubCommandScan,
    utils::{select_base_module, Spinner},
};

impl SubCommandScan {
    pub fn init(self) -> Result<(), super::error::Error> {
        let SubCommandScan { ref file, target, depth, offset, node, dir } = self;

        let mut spinner = Spinner::start("Start loading cache...");
        let ptrsx = PtrsxScanner::new(file)?;
        spinner.stop("cache loaded.");

        let pages = select_base_module(ptrsx.pages())?;
        let mut spinner = Spinner::start("Start creating pointer maps...");
        let rev_map = ptrsx.get_rev_pointer_map();
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
                    base: base as usize,
                    depth,
                    target: target.0,
                    node,
                    range: offset.0,
                    points: &points,
                    writer: &mut BufWriter::new(file),
                };
                ptrsx.scan(&rev_map, params)
            })?;
        spinner.stop("Pointer chain is scanned.");

        Ok(())
    }
}
